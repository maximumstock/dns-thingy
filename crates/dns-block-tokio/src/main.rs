mod cli;
mod recording;
mod resolution;

use cli::ServerArgs;
use resolution::{handle_benchmark, handle_filter, RequestAssociationMap, RequestKey};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, thread::available_parallelism};
use tokio::{net::UdpSocket, sync::Mutex, time::Instant};

use dns::{
    filter::is_domain_blacklisted,
    parser::{DnsPacketBuffer, DnsParser},
    resolver::relay_query_async,
};

#[tokio::main]
async fn main() {
    let server_args = ServerArgs::from_env();

    println!(
        "Started DNS blocker on {0}::{1} [benchmark={2}]",
        server_args.bind_address, server_args.bind_port, server_args.benchmark,
    );
    println!("Options {server_args:#?}");

    println!(
        "Number of Cores: {0}",
        available_parallelism().unwrap().get()
    );

    // TODO: benchmark this at some point
    // A) Create a pool of tasks to handle incoming DNS requests
    // start_server_without_task_delegation(server_args.clone()).await;
    // B) One acceptor task that spawns further tasks for each incoming request
    // start_server_with_acceptors(server_args.clone(), 1).await;
    // C) Multiple acceptor tasks that spawn further tasks for each incoming request
    start_server_with_acceptors(server_args, get_acceptor_pool_size()).await;
}

async fn start_server_with_acceptors(server_args: ServerArgs, num_acceptor_tasks: u8) {
    let server_args = Arc::new(server_args);
    let socket = Arc::new(
        tokio::net::UdpSocket::bind((server_args.bind_address.clone(), server_args.bind_port))
            .await
            .unwrap(),
    );
    let upstream_socket = Arc::new(tokio::net::UdpSocket::bind(("0.0.0.0", 0)).await.unwrap());
    let request_associations = Arc::new(Mutex::new(HashMap::new()));

    let mut handles = vec![];
    for _ in 0..num_acceptor_tasks {
        let server_args = Arc::clone(&server_args);
        let socket = Arc::clone(&socket);
        let upstream_socket = Arc::clone(&upstream_socket);
        let request_associations = Arc::clone(&request_associations);

        let handle = tokio::spawn(async move {
            loop {
                let server_args = Arc::clone(&server_args);
                let socket = Arc::clone(&socket);
                let upstream_socket = Arc::clone(&upstream_socket);
                let request_associations = Arc::clone(&request_associations);

                let mut buffer = [0u8; 512];
                let (_, sender) = socket.recv_from(&mut buffer).await.unwrap();

                tokio::spawn(async move {
                    process(
                        &socket,
                        &upstream_socket,
                        &buffer,
                        &sender,
                        &server_args,
                        request_associations,
                    )
                    .await;
                });
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

fn get_acceptor_pool_size() -> u8 {
    available_parallelism().unwrap().get() as u8 / 2
}

async fn process(
    receiving_socket: &UdpSocket,
    upstream_socket: &UdpSocket,
    original_query: &DnsPacketBuffer,
    sender: &SocketAddr,
    server_args: &ServerArgs,
    request_associations: Arc<Mutex<RequestAssociationMap>>,
) {
    let mut parser = DnsParser::new(original_query);
    let request_packet = parser.parse().unwrap();

    if server_args.benchmark {
        handle_benchmark(
            request_packet.header.request_id,
            receiving_socket,
            sender,
            std::time::Duration::from_millis(server_args.resolution_delay_ms),
        )
        .await;
    } else if is_domain_blacklisted(&request_packet.question.domain_name) {
        handle_filter(server_args, &request_packet, receiving_socket, sender).await;
    } else {
        let start = Instant::now();
        // Create a unqiue key that identifies the query, store it in a shared hashmap and
        // pass it to `handle_resolution` so it can later lookup who to send it to.
        let sender_key = RequestKey::from_packet(&request_packet);

        request_associations
            .lock()
            .await
            .insert(sender_key.clone(), (*sender, request_packet, start));

        async move {
            match relay_query_async(original_query, &server_args.dns_relay, upstream_socket).await {
                Ok(reply_buffer) => {
                    let reply_packet = DnsParser::new(&reply_buffer).parse().unwrap();
                    let request_key = RequestKey::from_packet(&reply_packet);
                    let request_data = request_associations.lock().await.remove(&request_key);

                    match request_data {
                        Some((client_address, request_packet, started_at)) => {
                            debug_assert_eq!(
                                reply_packet.question.domain_name,
                                request_packet.question.domain_name
                            );
                            debug_assert_eq!(
                                reply_packet.header.request_id,
                                request_packet.header.request_id
                            );

                            // Send the upstream DNS reply to the original client that sent the DNS query.
                            // We need to use the same client that we used to accept the client's query, so that
                            // the client does not invalidate our response because of a port mismatch, since
                            // any other socket would not be on the DNS listening port.
                            receiving_socket
                                .send_to(&reply_buffer, client_address)
                                .await
                                .unwrap();

                            // todo: record handled response metric

                            if !server_args.quiet {
                                println!(
                                    "Handled {:?} query for {} [{}ms]",
                                    &reply_packet.question.r#type,
                                    &reply_packet.question.domain_name,
                                    started_at.elapsed().as_millis()
                                );
                            }
                        }
                        None => {
                            eprintln!("No matching sender address for {:?}", request_key);
                        }
                    }
                }
                Err(e) => {
                    dbg!(e);
                }
            }
        }
        .await;
    }
}

// #[allow(unused)]
// async fn start_server_without_task_delegation(server_args: ServerArgs) {
//     let server_args = Arc::new(server_args);
//     let socket = Arc::new(
//         tokio::net::UdpSocket::bind((server_args.bind_address.clone(), server_args.bind_port))
//             .await
//             .unwrap(),
//     );

//     let mut handles = vec![];
//     for _ in 0..get_acceptor_pool_size() {
//         let server_args = Arc::clone(&server_args);
//         let socket = Arc::clone(&socket);

//         let handle = tokio::spawn(async move {
//             loop {
//                 let mut buffer = [0u8; 512];
//                 let (_, sender) = socket.recv_from(&mut buffer).await.unwrap();

//                 process(&socket, &buffer, &sender, &server_args).await;
//             }
//         });
//         handles.push(handle);
//     }

//     for handle in handles {
//         handle.await.unwrap();
//     }
// }

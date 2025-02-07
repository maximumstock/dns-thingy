mod cache;
mod cli;
mod recording;
mod resolution;

use cache::{CacheKey, RequestCache};
use cli::ServerArgs;
use resolution::{handle_benchmark, handle_filter, RequestAssociationMap, RequestKey};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, thread::available_parallelism};
use tokio::{net::UdpSocket, sync::RwLock, time::Instant};

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
    let request_associations = Arc::new(RwLock::new(HashMap::new()));
    let request_cache = Arc::new(RwLock::new(RequestCache::new()));

    let mut handles = vec![];
    for _ in 0..num_acceptor_tasks {
        let server_args = Arc::clone(&server_args);
        let socket = Arc::clone(&socket);
        let upstream_socket = Arc::clone(&upstream_socket);
        let request_associations = Arc::clone(&request_associations);
        let request_cache = Arc::clone(&request_cache);

        let handle = tokio::spawn(async move {
            loop {
                let server_args = Arc::clone(&server_args);
                let socket = Arc::clone(&socket);
                let upstream_socket = Arc::clone(&upstream_socket);
                let request_associations = Arc::clone(&request_associations);
                let request_cache = Arc::clone(&request_cache);

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
                        request_cache,
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

/// Returns the number of acceptor Tokio tasks to spawn, based on the number
/// of CPU cores that this application runs on.
fn get_acceptor_pool_size() -> u8 {
    available_parallelism().unwrap().get() as u8 / 2
}

async fn process(
    receiving_socket: &UdpSocket,
    relay_socket: &UdpSocket,
    client_packet: &DnsPacketBuffer,
    sender: &SocketAddr,
    server_args: &ServerArgs,
    request_associations: Arc<RwLock<RequestAssociationMap>>,
    request_cache: Arc<RwLock<RequestCache>>,
) {
    let mut parser = DnsParser::new(client_packet);
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

        let cache_key = CacheKey::from_packet(&request_packet);
        if let Some(dns_reply) = request_cache
            .write()
            .await
            .get(cache_key.clone(), request_packet.header.request_id)
        {
            receiving_socket.send_to(&dns_reply, sender).await.unwrap();

            // todo: record cache hit

            if !server_args.quiet {
                println!(
                    "[Cache Hit] Handled {:?} query for {} [{}ms]",
                    &request_packet.question.r#type,
                    &request_packet.question.domain_name,
                    start.elapsed().as_millis()
                );
            }

            return;
        }

        // Create a unqiue key that identifies the query, store it in a shared hashmap and
        // pass it to `handle_resolution` so it can later lookup who to send it to.
        let sender_key = RequestKey::from_packet(&request_packet);

        request_associations.write().await.insert(
            sender_key.clone(),
            (*sender, request_packet, start, cache_key),
        );

        // We send the incoming client DNS packet to the configured relay DNS server via `relay_socket` and get back
        // a DNS response as a raw `DnsPacketBuffer` or an error.
        match relay_query_async(client_packet, &server_args.dns_relay, relay_socket).await {
            Ok(reply_buffer) => {
                // todo: limit the number of times we need to parse DNS packets
                let reply_packet = DnsParser::new(&reply_buffer).parse().unwrap();
                let unique_request_key = RequestKey::from_packet(&reply_packet);
                let request_data = request_associations
                    .write()
                    .await
                    .remove(&unique_request_key);

                match request_data {
                    Some((client_address, request_packet, started_at, cache_key)) => {
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

                        request_cache.write().await.set(cache_key, reply_buffer);

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
                        eprintln!("No matching sender address for {:?}", unique_request_key);
                    }
                }
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }
}

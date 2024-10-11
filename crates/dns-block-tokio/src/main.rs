mod cli;
mod recording;
mod resolution;

use cli::ServerArgs;
use resolution::{handle_benchmark, handle_filter, handle_resolution};
use std::{sync::Arc, thread::available_parallelism};
use tokio::net::UdpSocket;

use dns::{
    filter::is_domain_blacklisted,
    parse::parser::{DnsPacketBuffer, DnsParser},
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

    // A) Create a pool of tasks to handle incoming DNS requests
    // start_server_without_task_delegation(server_args.clone()).await;
    // B) One acceptor task that spawns further tasks for each incoming request
    // start_server_with_acceptors(server_args.clone(), 1).await;
    // C) Multiple acceptor tasks that spawn further tasks for each incoming request
    start_server_with_acceptors(server_args, get_acceptor_pool_size()).await;
}

#[allow(unused)]
async fn start_server_without_task_delegation(server_args: ServerArgs) {
    let server_args = Arc::new(server_args);
    let socket = Arc::new(
        tokio::net::UdpSocket::bind((server_args.bind_address.clone(), server_args.bind_port))
            .await
            .unwrap(),
    );
    let sender_socket = Arc::new(tokio::net::UdpSocket::bind("0.0.0.0").await.unwrap());

    let mut handles = vec![];
    for _ in 0..get_acceptor_pool_size() {
        let server_args = Arc::clone(&server_args);

        let socket = Arc::clone(&socket);
        let sender_socket = Arc::clone(&sender_socket);
        let handle = tokio::spawn(async move {
            loop {
                let mut buffer = [0u8; 512];
                let (_, sender) = socket.recv_from(&mut buffer).await.unwrap();

                process(&socket, &sender_socket, &buffer, &sender, &server_args).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

async fn start_server_with_acceptors(server_args: ServerArgs, num_acceptor_tasks: u8) {
    let server_args = Arc::new(server_args);
    let socket = Arc::new(
        tokio::net::UdpSocket::bind((server_args.bind_address.clone(), server_args.bind_port))
            .await
            .unwrap(),
    );
    let sender_socket = Arc::new(tokio::net::UdpSocket::bind(("0.0.0.0", 0)).await.unwrap());

    let mut handles = vec![];
    for _ in 0..num_acceptor_tasks {
        let server_args = Arc::clone(&server_args);
        let socket = Arc::clone(&socket);
        let sender_socket = Arc::clone(&sender_socket);

        let handle = tokio::spawn(async move {
            loop {
                let server_args = Arc::clone(&server_args);
                let socket = Arc::clone(&socket);
                let sender_socket = Arc::clone(&sender_socket);

                let mut buffer = [0u8; 512];
                let (_, sender) = socket.recv_from(&mut buffer).await.unwrap();

                tokio::spawn(async move {
                    process(&socket, &sender_socket, &buffer, &sender, &server_args).await;
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
    sender: &std::net::SocketAddr,
    server_args: &ServerArgs,
) {
    let start = std::time::SystemTime::now();
    let mut parser = DnsParser::new(original_query);
    let (request_id, question) = parser.get_relay_information().unwrap();

    if server_args.benchmark {
        handle_benchmark(
            request_id,
            receiving_socket,
            sender,
            std::time::Duration::from_millis(server_args.resolution_delay_ms),
        )
        .await;
    } else if is_domain_blacklisted(&question.domain_name) {
        handle_filter(server_args, &question, request_id, receiving_socket, sender).await;
    } else {
        handle_resolution(
            original_query,
            server_args,
            receiving_socket,
            upstream_socket,
            sender,
            start,
        )
        .await;
    }
}

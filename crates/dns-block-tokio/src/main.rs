mod cli;
mod recording;

use cli::ServerArgs;
use std::{sync::Arc, thread::available_parallelism, time::Duration};
use tokio::net::UdpSocket;

use dns::{
    dns::{generate_response, DnsPacketBuffer},
    filter::is_domain_blacklisted,
    resolver::{extract_dns_question, resolve_domain_async, stub_response_with_delay},
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

    let mut handles = vec![];
    for _ in 0..get_acceptor_pool_size() {
        let server_args = Arc::clone(&server_args);

        let socket = Arc::clone(&socket);
        let handle = tokio::spawn(async move {
            loop {
                let mut buffer = [0u8; 512];
                let (_, sender) = socket.recv_from(&mut buffer).await.unwrap();

                process(&socket, &buffer, &sender, &server_args).await;
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

    let mut handles = vec![];
    for _ in 0..num_acceptor_tasks {
        let server_args = Arc::clone(&server_args);
        let socket = Arc::clone(&socket);

        let handle = tokio::spawn(async move {
            loop {
                let server_args = Arc::clone(&server_args);
                let socket = Arc::clone(&socket);

                let mut buffer = [0u8; 512];
                let (_, sender) = socket.recv_from(&mut buffer).await.unwrap();

                tokio::spawn(async move {
                    process(&socket, &buffer, &sender, &server_args).await;
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
    available_parallelism().unwrap().get() as u8
}

async fn process(
    socket: &UdpSocket,
    buf: DnsPacketBuffer<'_>,
    sender: &std::net::SocketAddr,
    server_args: &ServerArgs,
) {
    let start = std::time::SystemTime::now();
    let question = extract_dns_question(buf).unwrap();

    if server_args.benchmark {
        handle_benchmark(question.request_id, socket, sender).await;
    } else if is_domain_blacklisted(&question.domain_name) {
        handle_filter(server_args, &question, socket, sender).await
    } else {
        handle_resolution(question, server_args, socket, sender, start).await
    }
}

async fn handle_resolution(
    question: dns::dns::Question,
    server_args: &ServerArgs,
    socket: &tokio::net::UdpSocket,
    sender: &std::net::SocketAddr,
    start: std::time::SystemTime,
) {
    match resolve_domain_async(
        &question.domain_name,
        &server_args.dns_relay,
        Some(question.request_id),
        None,
    )
    .await
    {
        Ok((_, reply)) => {
            socket.send_to(&reply, sender).await.unwrap();
            if !server_args.quiet {
                println!(
                    "Handled query for {} [{}ms]",
                    question.domain_name,
                    std::time::SystemTime::now()
                        .duration_since(start)
                        .unwrap()
                        .as_millis()
                );
            }
        }
        Err(e) => {
            dbg!(e);
        }
    }
}

async fn handle_filter(
    server_args: &ServerArgs,
    question: &dns::dns::Question,
    socket: &tokio::net::UdpSocket,
    sender: &std::net::SocketAddr,
) {
    if !server_args.quiet {
        println!("Blocking request for {:?}", question.domain_name);
    }
    let nx_response =
        generate_response(question.request_id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
    socket.send_to(&nx_response, sender).await.unwrap();
}

async fn handle_benchmark(
    request_id: u16,
    socket: &tokio::net::UdpSocket,
    sender: &std::net::SocketAddr,
) {
    // If we are benchmarking, we don't need to resolve the domain and instead send a canned response
    let benchmark_resolution_delay = Duration::from_millis(5);
    let (_, reply) = stub_response_with_delay(Some(request_id), benchmark_resolution_delay)
        .await
        .unwrap();
    socket.send_to(&reply, sender).await.unwrap();
}

mod cache;
mod cli;
mod recording;
mod resolution;

use cli::ServerArgs;
use resolution::Resolver;
use std::{sync::Arc, thread::available_parallelism};
use tokio::task::JoinHandle;

use dns::parser::DnsPacketBuffer;

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
    let client_socket =
        tokio::net::UdpSocket::bind((server_args.bind_address.clone(), server_args.bind_port))
            .await
            .unwrap();
    let upstream_socket = tokio::net::UdpSocket::bind(("0.0.0.0", 0)).await.unwrap();
    let resolver = Arc::new(Resolver::new(server_args, client_socket, upstream_socket));

    let acceptor_task_handles: Vec<JoinHandle<_>> = (0..num_acceptor_tasks)
        .map(|_| {
            let resolver = Arc::clone(&resolver);

            tokio::spawn(async move {
                // Every acceptor task blocks and waits for the next UDP packet to come in...
                loop {
                    let resolver = Arc::clone(&resolver);
                    let mut buffer: DnsPacketBuffer = [0u8; 512];
                    let (_, sender) = resolver.client_socket.recv_from(&mut buffer).await.unwrap();

                    // ...and then dispatches processing that UDP packet to an independent Tokio task, so that accepting and processing
                    // are decoupled and we don't block accepting new incoming UDP packets from being processed
                    tokio::spawn(async move {
                        resolver.process(&buffer, &sender).await;
                    });
                }
            })
        })
        .collect();

    for handle in acceptor_task_handles {
        handle.await.unwrap();
    }
}

/// Returns the number of acceptor Tokio tasks to spawn, based on the number
/// of CPU cores that this application runs on.
fn get_acceptor_pool_size() -> u8 {
    available_parallelism().unwrap().get() as u8 / 2
}

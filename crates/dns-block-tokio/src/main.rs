mod cache;
mod cli;
mod recording;
mod resolution;

use cli::ServerArgs;
use futures::stream::{self, StreamExt};
use http::{HeaderValue, Uri};
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

async fn start_server_with_acceptors(mut server_args: ServerArgs, num_acceptor_tasks: u8) {
    let client_socket =
        tokio::net::UdpSocket::bind((server_args.bind_address.clone(), server_args.bind_port))
            .await
            .unwrap();
    let upstream_socket = tokio::net::UdpSocket::bind(("0.0.0.0", 0)).await.unwrap();

    // Extend the explicitly passed list of blocked domains with blocked domains fetched from remote repositories
    // TODO: what about wildcard domain blocklist entries? Should *.<domain>.<tld> block any subdomain of <domain>.<tld>
    // or should blocking <domain>.<tld> just imply blocking any subdomain as well?
    let external_blocked_domains: Vec<String> =
        resolve_external_blocked_domains(&server_args.domain_blacklists).await;
    server_args
        .blocked_domains
        .extend_from_slice(&external_blocked_domains);

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

async fn resolve_external_blocked_domains(domain_blacklist_uris: &[String]) -> Vec<String> {
    let valid_uris = domain_blacklist_uris
        .iter()
        .flat_map(|string| string.parse::<Uri>());

    let successful_response_bodies = stream::iter(valid_uris)
        .map(|uri| reqwest::get(uri.to_string()))
        .filter_map(async move |res| {
            let res = res.await;

            match res {
                Ok(response) => {
                    if !response.status().is_success()
                        || !response
                            .headers()
                            .get("content-type")
                            .unwrap_or(&HeaderValue::from_static("text/plain"))
                            .to_str()
                            .unwrap()
                            .starts_with("text/plain")
                    {
                        eprintln!(
                            "Fetching blocked domains from {} failed",
                            response.url().as_str()
                        );
                        return None;
                    }

                    let url = response.url().as_str().to_string();
                    let body = response.text().await;

                    match body {
                        Ok(text) => Some(async { (url, text) }),
                        Err(err) => {
                            eprintln!(
                                "Fetching blocked domains from {} failed: {:?}",
                                err.url().unwrap(),
                                err
                            );
                            None
                        }
                    }
                }
                Err(err) => {
                    eprintln!(
                        "Fetching blocked domains from {} failed: {:?}",
                        err.url().unwrap(),
                        err
                    );
                    None
                }
            }
        })
        .buffer_unordered(5)
        .collect::<Vec<_>>()
        .await;

    let domains = successful_response_bodies
        .into_iter()
        .flat_map(|(url, body)| {
            let domains = body
                .split('\n')
                // Some blocklists have comments in them and should not be empty
                .filter(|line| !line.starts_with('#') && line.len() > 3 && line.contains('.'))
                .map(|s| s.to_string())
                .collect::<Vec<_>>();

            println!(
                "[Debug] Loaded {} blocked domains from {}",
                domains.len(),
                url
            );

            domains
        })
        .collect::<Vec<_>>();

    domains.iter().map(|s| s.to_string()).collect()
}

/// Returns the number of acceptor Tokio tasks to spawn, based on the number
/// of CPU cores that this application runs on.
fn get_acceptor_pool_size() -> u8 {
    available_parallelism().unwrap().get() as u8 / 2
}

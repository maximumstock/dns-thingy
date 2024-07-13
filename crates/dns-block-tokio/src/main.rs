mod cli;

use cli::ServerArgs;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};
use tokio::{io::AsyncWriteExt, net::UdpSocket, sync::RwLock};

use dns::{
    dns::generate_response,
    filter::is_domain_blacklisted,
    resolver::{extract_query_id_and_domain, resolve_domain_async, stub_response_with_delay},
};

#[tokio::main]
async fn main() {
    let server_args = Arc::new(ServerArgs::from_env());

    let socket = UdpSocket::bind(("0.0.0.0", server_args.port))
        .await
        .unwrap();
    let socket = Arc::new(socket);

    println!(
        "Started DNS blocker on 127.0.0.1::{0} [benchmark={1}]",
        server_args.port, server_args.benchmark,
    );
    println!("Options {server_args:#?}");

    let query_recorder = setup_query_recorder(&server_args.recording_folder).await;

    // Potential Speed Improvements to tests:
    // - start by setting up N tasks on startup instead of waiting for a packet to create a task
    // - use a low-level socket API to configure socket settings (see Socket2)
    // - create a separate socket instance per task
    // - we don't need to parse the response DNS packet in the resolution function, as the parsing is only for displaying
    //   the results to the user. We can remove the parsing and just return the raw response packet.

    // But before that, we need to refactor the code to make it more testable
    // by implementing different setup strategies on startup.
    // Also, we need to add a resolution flag that bypasses the DNS resolution and
    // returns a canned response to speed up tests.

    loop {
        let socket = Arc::clone(&socket);
        let server_args = Arc::clone(&server_args);
        let query_recorder = Arc::clone(&query_recorder);

        let mut buf = [0; 512];
        // Bottleneck #1: we only ever read new requests one at a time, even though we use async IO
        let (_, sender) = socket.recv_from(&mut buf).await.unwrap();

        if let Some(ref f) = *query_recorder {
            f.write().await.write_all(&buf).await.unwrap();
        }

        tokio::spawn(async move {
            process(&socket, &buf, sender, &server_args).await;
        });
    }
}

async fn process(
    socket: &tokio::net::UdpSocket,
    buf: &[u8; 512],
    sender: SocketAddr,
    server_args: &ServerArgs,
) {
    let start = std::time::SystemTime::now();
    let (request_id, question) = extract_query_id_and_domain(buf).unwrap();

    if server_args.benchmark {
        handle_benchmark(request_id, socket, sender).await;
    } else if is_domain_blacklisted(&question.domain_name) {
        handle_filter(server_args, &question, request_id, socket, sender).await;
    } else {
        handle_resolution(question, server_args, request_id, socket, sender, start).await;
    }
}

async fn handle_resolution(
    question: dns::dns::Question,
    server_args: &ServerArgs,
    request_id: u16,
    socket: &UdpSocket,
    sender: SocketAddr,
    start: std::time::SystemTime,
) {
    match resolve_domain_async(
        &question.domain_name,
        &server_args.dns_relay,
        Some(request_id),
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
    request_id: u16,
    socket: &UdpSocket,
    sender: SocketAddr,
) {
    if !server_args.quiet {
        println!("Blocking request for {:?}", question.domain_name);
    }
    let nx_response = generate_response(request_id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
    socket.send_to(&nx_response, sender).await.unwrap();
}

async fn handle_benchmark(request_id: u16, socket: &UdpSocket, sender: SocketAddr) {
    // If we are benchmarking, we don't need to resolve the domain and instead send a canned response
    let benchmark_resolution_delay = Duration::from_millis(5);
    let (_, reply) = stub_response_with_delay(Some(request_id), benchmark_resolution_delay)
        .await
        .unwrap();
    socket.send_to(&reply, sender).await.unwrap();
}

async fn setup_query_recorder(
    record_query_path: &Option<String>,
) -> Arc<Option<RwLock<tokio::fs::File>>> {
    // TODO: avoid locks; write buffers to a dedicated tokio task via channels
    // which writes out the data
    if let Some(path) = record_query_path {
        tokio::fs::create_dir_all(Path::new(path)).await.unwrap();
        println!("Recording queries to {path}");
    }

    Arc::new({
        if let Some(ref path) = record_query_path {
            let filename = format!(
                "{}.bin",
                std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
            Some(RwLock::new(
                tokio::fs::File::create(PathBuf::new().join(path).join(filename))
                    .await
                    .unwrap(),
            ))
        } else {
            None
        }
    })
}

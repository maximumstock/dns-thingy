use clap::Parser;
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};
use tokio::{io::AsyncWriteExt, net::UdpSocket, sync::RwLock};

use dns::{
    dns::generate_response,
    filter::apply_domain_filter,
    resolver::{extract_query_id_and_domain, resolve_domain_async, resolve_domain_async_benchmark},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct ServerArgs {
    /// DNS server to forward to
    #[arg(short, long, default_value_t = String::from("1.1.1.1:53"))]
    dns: String,

    /// Port to listen on
    #[arg(short, long, default_value_t = 53000)]
    port: u16,

    /// Whether benchmark mode is enabled, ie. if forwarding should be skipped and to avoid network calls upstream
    #[arg(short, long, default_value_t = false)]
    benchmark: bool,

    /// Folder path to save DNS query recordings to
    #[arg(short, long)]
    recording_folder: Option<String>,

    /// Whether to disable logging
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
}

#[tokio::main]
async fn main() {
    let server_args = Arc::new(ServerArgs::parse());
    let socket = Arc::new(
        UdpSocket::bind(("0.0.0.0", server_args.port))
            .await
            .unwrap(),
    );
    let dns = Arc::new(server_args.dns.clone());

    println!(
        "Started DNS blocker on 127.0.0.1::{0} [benchmark={1}]",
        server_args.port, server_args.benchmark,
    );
    println!("Options {:#?}", server_args);

    let query_recorder = setup_query_recorder(&server_args.recording_folder).await;

    loop {
        let socket = Arc::clone(&socket);
        let dns = Arc::clone(&dns);
        let server_args = Arc::clone(&server_args);
        let query_recorder = Arc::clone(&query_recorder);

        let mut buf = [0; 512];
        let (_, sender) = socket.recv_from(&mut buf).await.unwrap();

        if let Some(ref f) = *query_recorder {
            f.write().await.write_all(&buf).await.unwrap();
        }

        tokio::spawn(async move {
            process(&socket, &dns, &buf, sender, &server_args).await;
        });
    }
}

async fn process(
    socket: &tokio::net::UdpSocket,
    dns: &str,
    buf: &[u8; 512],
    sender: SocketAddr,
    server_args: &ServerArgs,
) {
    let start = std::time::SystemTime::now();
    let (request_id, question) = extract_query_id_and_domain(buf).unwrap();

    if apply_domain_filter(&question.domain_name) {
        if !server_args.quiet {
            println!("Blocking request for {:?}", question.domain_name);
        }
        let nx_response = generate_response(request_id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
        socket.send_to(&nx_response, sender).await.unwrap();
    } else {
        if server_args.benchmark {
            let (_, reply) =
                resolve_domain_async_benchmark(&question.domain_name, dns, Some(request_id), None)
                    .await
                    .unwrap();
            socket.send_to(&reply, sender).await.unwrap();
            return;
        }
        match resolve_domain_async(&question.domain_name, dns, Some(request_id), None).await {
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
            Some(RwLock::new(
                tokio::fs::File::create(PathBuf::new().join(path).join(format!(
                    "{}.bin",
                    std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                )))
                .await
                .unwrap(),
            ))
        } else {
            None
        }
    })
}

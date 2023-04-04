use clap::Parser;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;

use dns::{
    dns::generate_response,
    filter::apply_domain_filter,
    resolver::{extract_query_id_and_domain, resolve_domain_async, resolve_domain_async_benchmark},
};

#[derive(Parser, Debug)]
// #[command(author, version, about, long_about = None)]
struct ServerArgs {
    #[arg(short, long, default_value_t = String::from("1.1.1.1:53"))]
    dns: String,
    #[arg(short, long, default_value_t = 53000)]
    port: u16,
    #[arg(short, long, default_value_t = false)]
    is_benchmark: bool,
    #[arg(short, long)]
    record_query_path: String,
}

#[tokio::main]
async fn main() {
    let server_args = ServerArgs::parse();
    let socket = Arc::new(
        UdpSocket::bind(("0.0.0.0", server_args.port))
            .await
            .unwrap(),
    );
    let dns = Arc::new(server_args.dns);

    println!(
        "Started DNS blocker on 127.0.0.1::{0} [benchmark={1}]",
        server_args.port, server_args.is_benchmark
    );

    loop {
        let socket = Arc::clone(&socket);
        let dns = Arc::clone(&dns);

        let mut buf = [0; 512];
        let (_, sender) = socket.recv_from(&mut buf).await.unwrap();

        tokio::spawn(async move {
            process(&socket, &dns, buf, sender, server_args.is_benchmark).await;
        });
    }
}

async fn process(
    socket: &tokio::net::UdpSocket,
    dns: &str,
    buf: [u8; 512],
    sender: SocketAddr,
    is_benchmark: bool,
) {
    let (request_id, question) = extract_query_id_and_domain(buf).unwrap();

    if apply_domain_filter(&question.domain_name) {
        println!("Blocking request for {:?}", question.domain_name);
        let nx_response = generate_response(request_id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
        socket.send_to(&nx_response, sender).await.unwrap();
    } else {
        if is_benchmark {
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
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }
}

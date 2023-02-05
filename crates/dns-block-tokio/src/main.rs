use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;

use dns::{
    dns::generate_response,
    filter::apply_domain_filter,
    resolver::{extract_query_id_and_domain, resolve_domain_async},
};

const DEFAULT_DNS: &str = "1.1.1.1:53";
const DEFAULT_PORT: &str = "53000";

#[tokio::main]
async fn main() {
    let dns = Arc::new(std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into()));
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.into())
        .parse()
        .expect("Port must be a number");
    let socket = Arc::new(UdpSocket::bind(("0.0.0.0", port)).await.unwrap());

    println!("Started DNS blocker on 127.0.0.1::{port}");

    loop {
        let mut buf = [0; 512];
        let (_, sender) = socket.recv_from(&mut buf).await.unwrap();

        let socket = Arc::clone(&socket);
        let dns = Arc::clone(&dns);

        tokio::task::spawn(async move {
            process(&socket, &dns, buf, sender).await.unwrap();
        });
    }
}

async fn process(
    socket: &tokio::net::UdpSocket,
    dns: &str,
    buf: [u8; 512],
    sender: SocketAddr,
) -> Result<(), ()> {
    let (request_id, question) = extract_query_id_and_domain(buf).unwrap();
    // todo implement async resolve
    if apply_domain_filter(&question.domain_name) {
        println!("Blocking request for {:?}", question.domain_name);
        let nx_response = generate_response(request_id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
        socket.send_to(&nx_response, sender).await.unwrap();
    } else {
        match resolve_domain_async(&question.domain_name, dns, Some(request_id), None).await {
            Ok((_, reply)) => {
                socket.send_to(&reply, sender).await.unwrap();
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }

    Ok(())
}

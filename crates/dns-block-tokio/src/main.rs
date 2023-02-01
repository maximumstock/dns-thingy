use std::{net::SocketAddr, sync::Arc};
use tokio::net::UdpSocket;

use dns::resolver::{parse_query, resolve};

const DEFAULT_DNS: &str = "1.1.1.1";
const DEFAULT_PORT: &str = "53000";

#[tokio::main]
async fn main() {
    let dns = Arc::new(std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into()));
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.into())
        .parse()
        .expect("Port must be a number");

    println!("Started DNS blocker on 127.0.0.1::{port}");

    let socket = UdpSocket::bind(("0.0.0.0", port)).await.unwrap();
    let dns = Arc::clone(&dns);

    loop {
        let mut buf = [0; 512];
        let (_, sender) = socket.recv_from(&mut buf).await.unwrap();
        handler(&socket, &dns, buf, sender).await;
    }
}

async fn handler(socket: &tokio::net::UdpSocket, dns: &str, buf: [u8; 512], sender: SocketAddr) {
    let (id, question) = parse_query(buf).unwrap();
    // todo implement async resolve
    // todo use non-blocking resolution socket, ie. tokio::net::UdpSocket
    match resolve(&question.domain_name, dns, Some(id), None) {
        Ok((_, reply)) => {
            socket.send_to(&reply, sender).await.unwrap();
        }
        Err(e) => {
            dbg!(e);
        }
    }
}

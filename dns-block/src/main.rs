use std::net::UdpSocket;

use dns::resolver::{parse_query, resolve};

const DEFAULT_DNS: &str = "1.1.1.1";
const PORT: u16 = 53;

fn main() {
    let dns = std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into());
    let socket = UdpSocket::bind(("127.0.0.1", PORT)).unwrap();

    loop {
        let mut buf = (0..512).into_iter().map(|_| 0).collect::<Vec<_>>();
        let (_, sender) = socket.recv_from(&mut buf).unwrap();
        let (id, question) = parse_query(buf).unwrap();
        let (_, buf) = resolve(&question.domain_name, &dns, Some(id)).unwrap();
        socket.send_to(&buf, sender).unwrap();
    }
}

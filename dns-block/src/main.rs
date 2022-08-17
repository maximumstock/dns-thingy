use std::net::UdpSocket;

use dns::{
    dns::generate_nxdomain_response,
    resolver::{parse_query, resolve},
};

const DEFAULT_DNS: &str = "1.1.1.1";
const PORT: u16 = 53000;

fn main() {
    let dns = std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into());
    let socket = UdpSocket::bind(("127.0.0.1", PORT)).unwrap();

    println!("Started DNS blocker on 127.0.0.1::{}", PORT);

    loop {
        let mut buf = (0..512).into_iter().map(|_| 0).collect::<Vec<_>>();
        let (_, sender) = socket.recv_from(&mut buf).unwrap();
        let (id, question) = parse_query(buf).unwrap();
        if question.domain_name.eq("google.de") {
            println!("Blocking request for {:?}", question.domain_name);
            let nx_response = generate_nxdomain_response(id).unwrap();
            socket.send_to(&nx_response, sender).unwrap();
        } else {
            println!("Allowing request for {:?}", question.domain_name);
            let (_, buf) = resolve(&question.domain_name, &dns, Some(id)).unwrap();
            socket.send_to(&buf, sender).unwrap();
        }
    }
}

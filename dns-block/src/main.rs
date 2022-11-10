use std::{net::UdpSocket, time::Duration};

use dns::{
    dns::generate_response,
    resolver::{parse_query, resolve},
};

const DEFAULT_DNS: &str = "1.1.1.1";
const DEFAULT_PORT: &str = "53000";

fn main() {
    let dns = std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into());
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.into())
        .parse()
        .expect("Port must be a number");
    let internal_socket = UdpSocket::bind(("0.0.0.0", port)).unwrap();
    let external_socket = UdpSocket::bind(("0.0.0.0", 0)).unwrap();
    external_socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    println!("Started DNS blocker on 127.0.0.1::{}", port);

    let mut buf = [0; 512];
    loop {
        buf.fill(0);
        let (_, sender) = internal_socket.recv_from(&mut buf).unwrap();
        let (id, question) = parse_query(buf).unwrap();
        if question.domain_name.eq("google.de") {
            println!("Blocking request for {:?}", question.domain_name);
            let nx_response = generate_response(id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
            internal_socket.send_to(&nx_response, sender).unwrap();
        } else {
            match resolve(
                &question.domain_name,
                &dns,
                Some(id),
                Some(external_socket.try_clone().unwrap()),
            ) {
                Ok((_, buf)) => {
                    internal_socket.send_to(&buf, sender).unwrap();
                }
                Err(e) => {
                    dbg!(e);
                }
            }
        }
    }
}

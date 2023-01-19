use std::{net::UdpSocket, time::Duration};

use dns::{
    dns::{generate_response, Question},
    resolver::{parse_query, resolve},
};

const DEFAULT_DNS: &str = "1.1.1.1";
const DEFAULT_PORT: &str = "53000";

fn main() {
    let dns_host = std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into());
    let dns_port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.into())
        .parse()
        .expect("Port must be a number");

    let incoming_socket = UdpSocket::bind(("0.0.0.0", dns_port)).unwrap();
    let outcoming_socket = UdpSocket::bind(("0.0.0.0", 0)).unwrap();
    outcoming_socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    println!("Started DNS blocker on 127.0.0.1::{}", dns_port);

    let mut buf = [0; 512];
    loop {
        buf.fill(0);
        let (_, sender) = incoming_socket.recv_from(&mut buf).unwrap();
        let (request_id, question) = parse_query(buf).unwrap();

        if apply_filter(&question) {
            println!("Blocking request for {:?}", question.domain_name);
            let nx_response =
                generate_response(request_id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
            incoming_socket.send_to(&nx_response, sender).unwrap();
        } else {
            // todo: pipe initial request through to avoid generating another DNS query packet
            match resolve(
                &question.domain_name,
                &dns_host,
                Some(request_id),
                Some(outcoming_socket.try_clone().unwrap()),
            ) {
                Ok((_, buf)) => {
                    incoming_socket.send_to(&buf, sender).unwrap();
                }
                Err(e) => {
                    // todo: send back correct response op code based on error
                    // ie. SERVFAIL
                    let res =
                        generate_response(request_id, dns::dns::ResponseCode::SERVFAIL).unwrap();
                    incoming_socket.send_to(&res, sender).unwrap();
                    dbg!(e);
                }
            }
        }
    }
}

fn apply_filter(question: &Question) -> bool {
    question.domain_name.eq("google.de")
}

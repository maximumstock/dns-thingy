use std::{net::UdpSocket, time::Duration};

use dns::{
    dns::{generate_response, Question},
    resolver::{parse_query, resolve_pipe},
};

const DEFAULT_UPSTREAM_DNS: &str = "1.1.1.1:53";
const DEFAULT_PORT: &str = "53000";

fn main() {
    let upstream_dns_host =
        std::env::var("UPSTREAM_DNS").unwrap_or_else(|_| DEFAULT_UPSTREAM_DNS.into());
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

    let mut incoming_query = [0; 512];
    loop {
        incoming_query.fill(0);
        if let Ok((_, sender)) = incoming_socket.recv_from(&mut incoming_query) {
            let (request_id, question) = parse_query(incoming_query).unwrap();

            if apply_filter(&question) {
                println!("Blocking request for {:?}", question.domain_name);
                let nx_response =
                    generate_response(request_id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
                incoming_socket.send_to(&nx_response, sender).unwrap();
            } else {
                match resolve_pipe(
                    &incoming_query,
                    &upstream_dns_host,
                    Some(outcoming_socket.try_clone().unwrap()),
                ) {
                    Ok((_, dns_response)) => {
                        incoming_socket.send_to(&dns_response, sender).unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error from upstream DNS {:?}", e);
                        generate_response(request_id, dns::dns::ResponseCode::SERVFAIL)
                            .map(|res| incoming_socket.send_to(&res, sender).unwrap())
                            .unwrap();
                    }
                }
            }
        }
    }
}

fn apply_filter(question: &Question) -> bool {
    question.domain_name.eq("google.de")
}

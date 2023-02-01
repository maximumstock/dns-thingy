use std::{net::UdpSocket, sync::Arc};

use dns::{
    dns::generate_response,
    filter::apply_domain_filter,
    resolver::{parse_query, resolve_pipe},
};

const DEFAULT_DNS: &str = "1.1.1.1";
const DEFAULT_PORT: &str = "53000";

fn main() {
    let dns = Arc::new(std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into()));
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.into())
        .parse()
        .expect("Port must be a number");
    let internal_socket = UdpSocket::bind(("0.0.0.0", port)).unwrap();
    let external_socket = UdpSocket::bind(("0.0.0.0", 0)).unwrap();

    println!("Started DNS blocker on 127.0.0.1::{port}");

    let mut handles = vec![];

    for _ in 0..4 {
        let internal_socket = internal_socket.try_clone().unwrap();
        let external_socket = external_socket.try_clone().unwrap();
        let dns = Arc::clone(&dns);
        let mut incoming_query = [0; 512];

        let handle = std::thread::spawn(move || loop {
            process(
                &internal_socket,
                &mut incoming_query,
                &dns,
                &external_socket,
            );
            incoming_query.fill(0);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn process(
    internal_socket: &UdpSocket,
    incoming_query: &mut [u8; 512],
    dns: &Arc<String>,
    external_socket: &UdpSocket,
) {
    let (_, sender) = internal_socket.recv_from(incoming_query).unwrap();
    let (request_id, question) = parse_query(*incoming_query).unwrap();

    if apply_domain_filter(&question.domain_name) {
        println!("Blocking request for {:?}", question.domain_name);
        let nx_response = generate_response(request_id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
        internal_socket.send_to(&nx_response, sender).unwrap();
    } else {
        match resolve_pipe(
            &*incoming_query,
            (dns.as_str(), 53),
            Some(external_socket.try_clone().unwrap()),
        ) {
            Ok((_, reply)) => {
                internal_socket.send_to(&reply, sender).unwrap();
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }
}

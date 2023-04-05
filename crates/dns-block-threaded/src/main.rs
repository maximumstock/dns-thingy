use std::{net::UdpSocket, sync::Arc};

use dns::{
    dns::generate_response,
    filter::apply_domain_filter,
    resolver::{extract_query_id_and_domain, resolve_domain, resolve_domain_benchmark},
};

const DEFAULT_DNS: &str = "1.1.1.1:53";
const DEFAULT_PORT: &str = "53000";

fn main() {
    let dns = Arc::new(std::env::var("DNS").unwrap_or_else(|_| DEFAULT_DNS.into()));
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| DEFAULT_PORT.into())
        .parse()
        .expect("Port must be a number");
    let is_benchmark: bool = std::env::var("DNS_BENCHMARK").is_ok();
    let internal_socket = UdpSocket::bind(("0.0.0.0", port)).unwrap();
    let external_socket = UdpSocket::bind(("0.0.0.0", 0)).unwrap();

    println!("Started DNS blocker on 127.0.0.1::{port} [benchmark={is_benchmark}]");

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
                is_benchmark,
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
    is_benchmark: bool,
) {
    let (_, sender) = internal_socket.recv_from(incoming_query).unwrap();
    let (request_id, question) = extract_query_id_and_domain(*incoming_query).unwrap();

    if apply_domain_filter(&question.domain_name) {
        println!("Blocking request for {:?}", question.domain_name);
        let nx_response = generate_response(request_id, dns::dns::ResponseCode::NXDOMAIN).unwrap();
        internal_socket.send_to(&nx_response, sender).unwrap();
    } else {
        if is_benchmark {
            let (_, reply) = resolve_domain_benchmark(
                &question.domain_name,
                dns.as_str(),
                Some(request_id),
                Some(external_socket.try_clone().unwrap()),
            )
            .unwrap();
            internal_socket.send_to(&reply, sender).unwrap();
            return;
        }
        match resolve_domain(
            &question.domain_name,
            dns.as_str(),
            Some(request_id),
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

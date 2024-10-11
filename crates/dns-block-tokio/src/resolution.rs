use dns::{
    parse::parser::{DnsPacketBuffer, DnsParser},
    protocol::{question::Question, utils::generate_nx_response},
    resolver::{relay_query_async, stub_response_with_delay},
};

use crate::cli::ServerArgs;

/// TODO: should have a sender socket so we don't re-bind on each upstream send in `resolve_domain_async`
pub async fn handle_resolution(
    query: &DnsPacketBuffer,
    server_args: &ServerArgs,
    receiving_socket: &tokio::net::UdpSocket,
    upstream_socket: &tokio::net::UdpSocket,
    sender: &std::net::SocketAddr,
    start: std::time::SystemTime,
) {
    match relay_query_async(query, &server_args.dns_relay, upstream_socket).await {
        Ok(reply) => {
            receiving_socket.send_to(&reply, sender).await.unwrap();
            if !server_args.quiet {
                // We only pick the first question, since multiple questions seem to be unsupported by most
                // nameservers anyways, see https://stackoverflow.com/questions/4082081/requesting-a-and-aaaa-records-in-single-dns-query/4083071#4083071.
                let (_, question) = DnsParser::new(query).get_relay_information().unwrap();
                println!(
                    "Handled query for {} [{}ms]",
                    &question.domain_name,
                    std::time::SystemTime::now()
                        .duration_since(start)
                        .unwrap()
                        .as_millis()
                );
            }
        }
        Err(e) => {
            dbg!(e);
        }
    }
}

pub async fn handle_filter(
    server_args: &ServerArgs,
    question: &Question,
    request_id: u16,
    socket: &tokio::net::UdpSocket,
    sender: &std::net::SocketAddr,
) {
    if !server_args.quiet {
        println!("Blocking request for {:?}", question.domain_name);
    }
    let nx_response = generate_nx_response(request_id).unwrap();
    socket.send_to(&nx_response, sender).await.unwrap();
}

pub async fn handle_benchmark(
    request_id: u16,
    socket: &tokio::net::UdpSocket,
    sender: &std::net::SocketAddr,
    resolution_delay: std::time::Duration,
) {
    let (_, reply) = stub_response_with_delay(Some(request_id), resolution_delay)
        .await
        .unwrap();
    socket.send_to(&reply, sender).await.unwrap();
}

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use dns::{
    parse::parser::{DnsPacket, DnsPacketBuffer, DnsParser},
    protocol::utils::generate_nx_response,
    resolver::{relay_query_async, stub_response_with_delay},
};
use tokio::{sync::Mutex, time::Instant};

use crate::cli::ServerArgs;

pub type RequestAssociationMap = HashMap<RequestKey, (SocketAddr, DnsPacket, Instant)>;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct RequestKey {
    record_type: usize,
    request_id: u16,
    domain: String,
}

impl RequestKey {
    pub fn new(record_type: usize, request_id: u16, domain: String) -> Self {
        RequestKey {
            record_type,
            request_id,
            domain,
        }
    }

    pub(crate) fn from_packet(packet: &dns::parse::parser::DnsPacket) -> Self {
        RequestKey::new(
            packet.question.r#type,
            packet.header.request_id,
            packet.question.domain_name.clone(),
        )
    }
}

pub async fn handle_resolution(
    query: &DnsPacketBuffer,
    server_args: &ServerArgs,
    receiving_socket: &tokio::net::UdpSocket,
    upstream_socket: &tokio::net::UdpSocket,
    request_associations: Arc<Mutex<RequestAssociationMap>>,
) {
    match relay_query_async(query, &server_args.dns_relay, upstream_socket).await {
        Ok(reply_buffer) => {
            let reply_packet = DnsParser::new(&reply_buffer).parse().unwrap();
            let request_key = RequestKey::from_packet(&reply_packet);
            let request_data = request_associations.lock().await.remove(&request_key);

            match request_data {
                Some((client_address, request_packet, started_at)) => {
                    debug_assert_eq!(
                        reply_packet.question.domain_name,
                        request_packet.question.domain_name
                    );
                    debug_assert_eq!(
                        reply_packet.header.request_id,
                        request_packet.header.request_id
                    );

                    // Send the upstream DNS reply to the original client that sent the DNS query.
                    // We need to use the same client that we used to accept the client's query, so that
                    // the client does not invalidate our response because of a port mismatch, since
                    // any other socket would not be on the DNS listening port.
                    receiving_socket
                        .send_to(&reply_buffer, client_address)
                        .await
                        .unwrap();

                    if !server_args.quiet {
                        println!(
                            "Handled query for {} [{}ms]",
                            &reply_packet.question.domain_name,
                            started_at.elapsed().as_millis()
                        );
                    }
                }
                None => {
                    eprintln!("No matching sender address for {:?}", request_key);
                }
            }
        }
        Err(e) => {
            dbg!(e);
        }
    }
}

pub async fn handle_filter(
    server_args: &ServerArgs,
    request_packet: &DnsPacket,
    socket: &tokio::net::UdpSocket,
    sender: &std::net::SocketAddr,
) {
    if !server_args.quiet {
        println!(
            "Blocking request for {:?}",
            request_packet.question.domain_name
        );
    }
    let nx_response = generate_nx_response(request_packet.header.request_id).unwrap();
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

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use dns::{
    parse::parser::{DnsPacket, DnsPacketBuffer, DnsParser},
    protocol::{record_type::RecordType, serialize::generate_nx_response},
    resolver::{relay_query_async, stub_response_with_delay},
};
use tokio::{sync::Mutex, time::Instant};

use crate::cli::ServerArgs;

pub type RequestAssociationMap = HashMap<RequestKey, (SocketAddr, DnsPacket, Instant)>;

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct RequestKey {
    record_type: RecordType,
    request_id: u16,
    domain: String,
}

impl RequestKey {
    pub fn new(record_type: RecordType, request_id: u16, domain: String) -> Self {
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

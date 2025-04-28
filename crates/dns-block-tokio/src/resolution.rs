use std::{
    collections::{BTreeSet, HashMap},
    net::SocketAddr,
    sync::Arc,
};

use dns::{
    parser::{DnsPacket, DnsPacketBuffer, DnsParser},
    protocol::record_type::RecordType,
    resolver::{relay_query_async, stub_response_with_delay},
    serialize::generate_nx_response,
};
use tokio::{net::UdpSocket, sync::RwLock, time::Instant};

use crate::{
    cache::{CacheKey, RequestCache},
    cli::ServerArgs,
};

pub struct Resolver {
    pub(crate) server_args: ServerArgs,
    pub(crate) request_associations: Arc<RwLock<RequestAssociationMap>>,
    pub(crate) request_cache: Arc<RwLock<RequestCache>>,
    pub(crate) blocked_domains: Arc<BTreeSet<String>>,
    pub(crate) client_socket: UdpSocket,
    pub(crate) upstream_socket: UdpSocket,
}

impl Resolver {
    pub fn new(
        server_args: ServerArgs,
        client_socket: UdpSocket,
        upstream_socket: UdpSocket,
    ) -> Self {
        Self {
            request_associations: Default::default(),
            request_cache: Arc::new(RwLock::new(RequestCache::new())),
            blocked_domains: Arc::new(BTreeSet::from_iter(server_args.blocked_domains.clone())),
            server_args,
            client_socket,
            upstream_socket,
        }
    }

    pub async fn process(&self, client_packet: &DnsPacketBuffer, sender: &SocketAddr) {
        let mut parser = DnsParser::new(client_packet);
        let request_packet = parser.parse().unwrap();

        if self.server_args.benchmark {
            handle_benchmark(
                request_packet.header.request_id,
                &self.client_socket,
                sender,
                std::time::Duration::from_millis(self.server_args.resolution_delay_ms),
            )
            .await;
        } else if self
            .blocked_domains
            .contains(&request_packet.question.domain_name)
        {
            handle_filter(
                &self.server_args,
                &request_packet,
                &self.client_socket,
                sender,
            )
            .await;
        } else {
            let start = Instant::now();

            let cache_key = CacheKey::from_packet(&request_packet);
            if self.server_args.caching_enabled {
                if let Some(dns_reply) = self
                    .request_cache
                    .write()
                    .await
                    .get(cache_key.clone(), request_packet.header.request_id)
                {
                    self.client_socket
                        .send_to(&dns_reply, sender)
                        .await
                        .unwrap();

                    // todo: record cache hit

                    if !self.server_args.quiet {
                        println!(
                            "[Cache Hit] Handled {:?} query for {} [{}ms]",
                            &request_packet.question.r#type,
                            &request_packet.question.domain_name,
                            start.elapsed().as_millis()
                        );
                    }

                    return;
                }
            }

            // Create a unqiue key that identifies the query, store it in a shared hashmap and
            // pass it to `handle_resolution` so it can later lookup who to send it to.
            let sender_key = RequestKey::from_packet(&request_packet);

            self.request_associations.write().await.insert(
                sender_key.clone(),
                (*sender, request_packet, start, cache_key),
            );

            // We send the incoming client DNS packet to the configured relay DNS server via `relay_socket` and get back
            // a DNS response as a raw `DnsPacketBuffer` or an error.
            match relay_query_async(
                client_packet,
                &self.server_args.dns_relay,
                &self.upstream_socket,
            )
            .await
            {
                Ok(reply_buffer) => {
                    // todo: limit the number of times we need to parse DNS packets
                    let reply_packet = DnsParser::new(&reply_buffer).parse().unwrap();
                    let unique_request_key = RequestKey::from_packet(&reply_packet);
                    let request_data = self
                        .request_associations
                        .write()
                        .await
                        .remove(&unique_request_key);

                    match request_data {
                        Some((client_address, request_packet, started_at, cache_key)) => {
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
                            self.client_socket
                                .send_to(&reply_buffer, client_address)
                                .await
                                .unwrap();

                            // todo: record handled response metric

                            if self.server_args.caching_enabled {
                                self.request_cache
                                    .write()
                                    .await
                                    .set(cache_key, reply_buffer);
                            }

                            if !self.server_args.quiet {
                                println!(
                                    "Handled {:?} query for {} [{}ms]",
                                    &reply_packet.question.r#type,
                                    &reply_packet.question.domain_name,
                                    started_at.elapsed().as_millis()
                                );
                            }
                        }
                        None => {
                            eprintln!("No matching sender address for {:?}", unique_request_key);
                        }
                    }
                }
                Err(e) => {
                    dbg!(e);
                }
            }
        }
    }
}

pub type RequestAssociationMap = HashMap<RequestKey, (SocketAddr, DnsPacket, Instant, CacheKey)>;

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

    pub(crate) fn from_packet(packet: &DnsPacket) -> Self {
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

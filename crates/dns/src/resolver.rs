use std::{net::UdpSocket, time::Duration};

use crate::{
    parser::{encode_domain_name, DnsPacketBuffer, DnsParser},
    protocol::answer::ResourceRecord,
    serialize::generate_nx_response,
};

/// Synchronously resolves INternet A records for `domain` using the DNS server `dns`
pub fn resolve_domain(
    domain: &str,
    dns: &str,
    id: Option<u16>,
    socket: Option<UdpSocket>,
) -> Result<(Vec<ResourceRecord>, [u8; 512]), Box<dyn std::error::Error + Send + Sync>> {
    let socket = socket.unwrap_or_else(|| UdpSocket::bind(("0.0.0.0", 0)).unwrap());

    let request = generate_request(domain, id);
    if let Err(e) = socket.send_to(&request, dns) {
        println!("Failed to send request for {domain} to {dns:?}: {e:?}");
        return Err(e.into());
    }

    let mut response = [0; 512];
    let (_, _) = socket.recv_from(&mut response).map_err(|e| {
        println!("Failed to receive response for {domain} from {dns:?}: {e:?}");
        e
    })?;

    let packet = DnsParser::new(&response).parse()?;
    Ok((packet.answers, response))
}

#[allow(unused)]
async fn resolve_domain_async(
    domain: &str,
    dns: &str,
    id: Option<u16>,
    socket: &tokio::net::UdpSocket,
) -> Result<(Vec<ResourceRecord>, [u8; 512]), Box<dyn std::error::Error + Send + Sync>> {
    let request = generate_request(domain, id);
    if let Err(e) = socket.send_to(&request, dns).await {
        println!("Failed to send request for {domain} to {dns:?}: {e:?}");
        return Err(e.into());
    }

    let mut response = [0; 512];
    let (_, _) = socket.recv_from(&mut response).await.map_err(|e| {
        println!("Failed to receive response for {domain} from {dns:?}: {e:?}");
        e
    })?;

    let packet = DnsParser::new(&response).parse()?;
    Ok((packet.answers, response))
}

/// Asynchronously send the incoming raw DNS packet to the relay DNS server and
/// pipes the response back to the originating socket.
pub async fn relay_query_async(
    original_query: &DnsPacketBuffer,
    upstream_dns: &str,
    socket: &tokio::net::UdpSocket,
) -> Result<DnsPacketBuffer, Box<dyn std::error::Error + Send + Sync>> {
    if let Err(e) = socket.send_to(original_query, upstream_dns).await {
        println!("Failed to send request to {upstream_dns:?}: {e:?}");
        return Err(e.into());
    }

    let mut response = [0; 512];
    let (_, _) = socket.recv_from(&mut response).await.map_err(|e| {
        println!("Failed to receive response from {upstream_dns:?}: {e:?}");
        e
    })?;

    Ok(response)
}

pub async fn stub_response_with_delay(
    id: Option<u16>,
    delay: Duration,
) -> Result<(Vec<ResourceRecord>, DnsPacketBuffer), Box<dyn std::error::Error + Send + Sync>> {
    let response = generate_nx_response(id.unwrap_or(1337)).unwrap();
    tokio::time::sleep(delay).await;
    // Still parse answers, to keep the same API as the actual resolve function
    let packet = DnsParser::new(&response).parse()?;
    Ok((packet.answers, response))
}

/// Generates a recursive DNS query for INternet A records
pub(crate) fn generate_request(domain: &str, id: Option<u16>) -> Vec<u8> {
    const DEFAULT_ID: [u8; 2] = [(1337u16 >> 4) as u8, (1337 & 0xFF) as u8];
    let id = id
        .map(|n| [(n >> 8) as u8, (n & 0xFF) as u8])
        .unwrap_or(DEFAULT_ID);
    const QTYPE: [u8; 2] = [0x00, 0x01];
    const QCLASS: [u8; 2] = [0x00, 0x01];

    let request_header: [u8; 12] = [
        id[0], id[1], // identification
        0x01, 0x00, // flags
        0x00, 0x01, // question section
        0x00, 0x00, // answer section
        0x00, 0x00, // authority section
        0x00, 0x00, // additional section
    ];
    let mut request = Vec::with_capacity(16 + domain.len());
    request.extend(request_header);
    request.extend(encode_domain_name(domain));
    request.extend(QTYPE);
    request.extend(QCLASS);
    request
}

#[cfg(test)]
mod tests {
    use crate::{protocol::answer::ResourceRecordData, resolver::resolve_domain};

    const DNS_SERVERS: [&str; 1] = ["1.1.1.1:53"];

    #[test]
    fn test_resolve_a_records() {
        for dns_root in DNS_SERVERS {
            let (answers, _) = resolve_domain("www.example.com", dns_root, None, None).unwrap();
            assert!(matches!(
                answers.last().unwrap().value,
                ResourceRecordData::A { ipv4: _ }
            ));
        }
    }
}

use crate::dns::{encode_domain_name, Answer, DnsParser, Question};

use std::net::UdpSocket;

/// Resolves INternet A records for `domain` using the DNS server `dns`
pub fn resolve_domain(
    domain: &str,
    dns: &str,
    id: Option<u16>,
    socket: Option<UdpSocket>,
) -> Result<(Vec<Answer>, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    let socket = socket.unwrap_or_else(|| UdpSocket::bind(("0.0.0.0", 0)).unwrap());

    let request = generate_request(domain, id);
    if let Err(e) = socket.send_to(&request, dns) {
        println!("Failed to send request for {domain} to {dns:?}: {e:?}");
        return Err(e.into());
    }

    let mut buffer = [0; 512].to_vec();
    let (datagram_size, _) = socket.recv_from(&mut buffer).map_err(|e| {
        println!("Failed to receive response for {domain} from {dns:?}: {e:?}");
        e
    })?;
    // buffer.truncate(datagram_size);

    parse_answers(buffer)
}

pub async fn resolve_domain_async(
    domain: &str,
    dns: &str,
    id: Option<u16>,
    existing_socket: Option<tokio::net::UdpSocket>,
) -> Result<(Vec<Answer>, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    let socket = if let Some(x) = existing_socket {
        x
    } else {
        tokio::net::UdpSocket::bind(("0.0.0.0", 0)).await.unwrap()
    };

    let request = generate_request(domain, id);
    if let Err(e) = socket.send_to(&request, dns).await {
        println!("Failed to send request for {domain} to {dns:?}: {e:?}");
        return Err(e.into());
    }

    let mut buffer = [0; 512].to_vec();
    let (datagram_size, _) = socket.recv_from(&mut buffer).await.map_err(|e| {
        println!("Failed to receive response for {domain} from {dns:?}: {e:?}");
        e
    })?;
    // buffer.truncate(datagram_size);

    parse_answers(buffer)
}

fn parse_answers(
    buffer: Vec<u8>,
) -> Result<(Vec<Answer>, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    let mut parser = DnsParser::new(buffer);
    let header = parser.parse_header();

    for _ in 0..header.question_count {
        parser.parse_question();
    }

    let answers = (0..header.answer_count)
        .map(|_| parser.parse_answer())
        .collect::<Vec<_>>();

    Ok((answers, parser.buf))
}

pub fn extract_query_id_and_domain(
    buf: [u8; 512],
) -> Result<(u16, Question), Box<dyn std::error::Error>> {
    let mut parser = DnsParser::new(buf.to_vec());
    let header = parser.parse_header();
    Ok((header.id, parser.parse_question()))
}

const DEFAULT_ID: (u8, u8) = ((1337u16 >> 4) as u8, (1337 & 0xFF) as u8);

/// Generates a recursive DNS query for INternet A records
pub(crate) fn generate_request(domain: &str, id: Option<u16>) -> Vec<u8> {
    let id = id
        .map(|n| ((n >> 8) as u8, (n & 0xFF) as u8))
        .unwrap_or(DEFAULT_ID);
    const QTYPE: [u8; 2] = [0x00, 0x01];
    const QCLASS: [u8; 2] = [0x00, 0x01];
    let request_header: [u8; 12] = [
        id.0, id.1, // identification
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
    use std::net::Ipv4Addr;

    use super::{resolve_domain, Answer};

    const DNS_SERVERS: [&str; 1] = ["1.1.1.1:53"];

    #[test]
    fn test_resolve_a_records() {
        for dns_root in DNS_SERVERS {
            let (answers, _) = resolve_domain("www.example.com", dns_root, None, None).unwrap();
            if let Some(Answer::A { ipv4, .. }) = answers.last() {
                assert_eq!(&Ipv4Addr::new(93, 184, 216, 34), ipv4);
            }

            let (answers, _) =
                resolve_domain("www.maximumstock.net", dns_root, None, None).unwrap();
            let expected = vec![Ipv4Addr::new(154, 53, 57, 10)];

            for answer in &answers {
                if let Answer::A { ipv4, .. } = answer {
                    assert!(expected.contains(ipv4));
                }
            }
        }
    }
}

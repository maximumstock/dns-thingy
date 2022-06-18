use crate::dns::{encode_domain_name, Answer, ResponseParser};

use std::{net::UdpSocket, time::Duration};

/// Resolves INternet A records for `domain` using the DNS server `dns`
pub fn resolve(domain: &str, dns: &str) -> Result<Vec<Answer>, Box<dyn std::error::Error>> {
    let request = generate_request(domain);

    let addr = (dns, 53);
    let socket = UdpSocket::bind(("0.0.0.0", 0))?;
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    socket.send_to(&request, &addr).unwrap();

    let mut buffer = (0..512).into_iter().map(|_| 0).collect::<Vec<_>>();
    let (datagram_size, _) = socket.recv_from(&mut buffer)?;
    buffer.truncate(datagram_size);

    let mut parser = ResponseParser::new(buffer);
    let header = parser.parse_header();

    for _ in 0..header.question_count {
        parser.parse_question();
    }

    let answers = (0..header.answer_count)
        .map(|_| parser.parse_answer())
        .collect::<Vec<_>>();

    assert!(parser.is_done());

    Ok(answers)
}

/// Generates a DNS query for INternet A records
pub(crate) fn generate_request(domain: &str) -> Vec<u8> {
    const QTYPE: [u8; 2] = [0x00, 0x01];
    const QCLASS: [u8; 2] = [0x00, 0x01];
    const REQUEST_HEADER: [u8; 12] = [
        0x10, 0x01, // identification
        0x01, 0x00, // flags
        0x00, 0x01, // question section
        0x00, 0x00, // answer section
        0x00, 0x00, // authority section
        0x00, 0x00, // additional section
    ];
    let mut request = vec![];
    request.extend(REQUEST_HEADER);
    request.extend(encode_domain_name(domain));
    request.extend(QTYPE);
    request.extend(QCLASS);
    request
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::{resolve, Answer};

    const DNS_SERVERS: [&str; 2] = ["192.168.178.31", "1.1.1.1"];

    #[test]
    fn test_resolve_a_records() {
        for dns_root in DNS_SERVERS {
            let answers = resolve("www.example.com", dns_root).unwrap();
            if let Some(Answer::A { ipv4, .. }) = answers.last() {
                assert_eq!(&Ipv4Addr::new(93, 184, 216, 34), ipv4);
            }

            let answers = resolve("www.wonder.me", dns_root).unwrap();
            let expected = vec![
                Ipv4Addr::new(52, 49, 198, 28),
                Ipv4Addr::new(52, 212, 43, 230),
                Ipv4Addr::new(3, 248, 8, 137),
            ];

            for answer in &answers {
                if let Answer::A { ipv4, .. } = answer {
                    assert!(expected.contains(ipv4));
                }
            }

            let answers = resolve("www.maximumstock.net", dns_root).unwrap();
            let expected = vec![Ipv4Addr::new(154, 53, 57, 10)];

            for answer in &answers {
                if let Answer::A { ipv4, .. } = answer {
                    assert!(expected.contains(ipv4));
                }
            }
        }
    }
}

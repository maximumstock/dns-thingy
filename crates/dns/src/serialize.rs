use std::net::Ipv4Addr;

use crate::{
    parser::{DnsPacketBuffer, encode_domain_name},
    protocol::{
        header::{Flags, Header},
        record_type::RecordType,
        response_code::ResponseCode,
    },
};

const CLASS_IN: u16 = 1;

/// Generates a response to an `A` query for `domain` with a single answer pointing at `ipv4`.
pub fn generate_a_response(
    id: u16,
    domain: &str,
    ipv4: Ipv4Addr,
    ttl: u32,
) -> Result<DnsPacketBuffer, Box<dyn std::error::Error>> {
    let flags = Flags {
        response_code: ResponseCode::NOERROR.into(),
        query: false,
        recursion_desired: true,
        recursion_available: true,
        ..Flags::default()
    };

    let header = Header {
        request_id: id,
        flags,
        question_count: 1,
        answer_count: 1,
        ..Header::default()
    };

    let record_type: u16 = RecordType::A.into();
    let h: [u8; 12] = header.into();

    let mut out = Vec::with_capacity(512);
    out.extend_from_slice(&h);
    // Question section
    out.extend(encode_domain_name(domain));
    out.extend_from_slice(&record_type.to_be_bytes());
    out.extend_from_slice(&CLASS_IN.to_be_bytes());
    // Answer section, whose name points back to the question's domain name at offset 12
    out.extend_from_slice(&[0xC0, 0x0C]);
    out.extend_from_slice(&record_type.to_be_bytes());
    out.extend_from_slice(&CLASS_IN.to_be_bytes());
    out.extend_from_slice(&ttl.to_be_bytes());
    out.extend_from_slice(&4u16.to_be_bytes());
    out.extend_from_slice(&ipv4.octets());

    if out.len() > 512 {
        return Err("generated A response exceeds 512 bytes".into());
    }

    let mut packet = [0u8; 512];
    packet[..out.len()].copy_from_slice(&out);
    Ok(packet)
}

pub fn generate_nx_response(id: u16) -> Result<DnsPacketBuffer, Box<dyn std::error::Error>> {
    let flags = Flags {
        response_code: ResponseCode::NXDOMAIN.into(),
        query: false,
        ..Flags::default()
    };

    let header = Header {
        request_id: id,
        flags,
        ..Header::default()
    };

    let mut packet = [0u8; 512];
    let h: [u8; 12] = header.into();
    packet[0..12].copy_from_slice(h.as_slice());
    Ok(packet)
}

pub fn generate_response_with_answer(
    id: u16,
    response_code: ResponseCode,
) -> Result<DnsPacketBuffer, Box<dyn std::error::Error>> {
    let flags = Flags {
        response_code: response_code.into(),
        query: false,
        ..Flags::default()
    };

    let header = Header {
        request_id: id,
        flags,
        ..Header::default()
    };

    let mut packet = Vec::with_capacity(512);
    let h: [u8; 12] = header.into();
    packet.extend_from_slice(h.as_slice());
    packet.extend_from_slice(&[0; 500]);
    Ok(packet.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use crate::{
        parser::DnsParser,
        protocol::{answer::ResourceRecordData, record_type::RecordType},
        serialize::generate_a_response,
    };

    #[test]
    fn test_generate_a_response() {
        let ip = Ipv4Addr::new(10, 0, 0, 5);
        let buf = generate_a_response(4242, "foo.example.com", ip, 60).unwrap();
        let packet = DnsParser::new(&buf).parse().unwrap();

        assert_eq!(packet.header.request_id, 4242);
        assert_eq!(packet.header.question_count, 1);
        assert_eq!(packet.header.answer_count, 1);
        assert_eq!(packet.question.domain_name, "foo.example.com");
        assert_eq!(packet.question.r#type, RecordType::A);
        assert_eq!(packet.answers.len(), 1);

        let answer = &packet.answers[0];
        assert_eq!(answer.meta.name, "foo.example.com");
        assert_eq!(answer.meta.record_type, RecordType::A);
        assert_eq!(answer.meta.ttl, 60);
        assert!(matches!(answer.value, ResourceRecordData::A { ipv4 } if ipv4 == ip));
    }
}

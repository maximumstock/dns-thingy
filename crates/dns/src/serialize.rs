use crate::{
    parser::DnsPacketBuffer,
    protocol::{
        header::{Flags, Header},
        response_code::ResponseCode,
    },
};

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

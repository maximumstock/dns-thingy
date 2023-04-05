use std::net::Ipv4Addr;

use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct DnsRequest {
    inner: Bytes,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Flags {
    pub query: bool,
    pub opcode: u8,
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub z: u8,
    pub response_code: u8,
}

impl From<u16> for Flags {
    fn from(input: u16) -> Self {
        Self {
            query: (input >> 15 & 1) == 0,
            opcode: (input >> 11 & 4) as u8,
            authoritative_answer: (input >> 10 & 1) > 0,
            truncation: (input >> 9 & 1) > 0,
            recursion_desired: (input >> 8 & 1) > 0,
            recursion_available: (input >> 7 & 1) > 0,
            z: (input >> 4 & 3) as u8,
            response_code: (input & 4) as u8,
        }
    }
}

impl From<Flags> for u16 {
    fn from(flags: Flags) -> Self {
        let mut value = 0u16;
        value |= if flags.query { 0 } else { 0x8000 }; // MSB needs to be set
        value |= (flags.opcode as u16) << 11;
        value |= u16::from(flags.authoritative_answer) << 10;
        value |= u16::from(flags.truncation) << 9;
        value |= u16::from(flags.recursion_desired) << 8;
        value |= u16::from(flags.recursion_available) << 7;
        value |= (flags.z as u16) << 3;
        value |= flags.response_code as u16;
        value
    }
}

#[derive(Debug)]
pub enum ResponseCode {
    NOERROR,
    FORMERR,
    NXDOMAIN,
    SERVFAIL,
}

impl From<ResponseCode> for u8 {
    fn from(rc: ResponseCode) -> Self {
        match rc {
            ResponseCode::NOERROR => 0,
            ResponseCode::FORMERR => 1,
            ResponseCode::SERVFAIL => 2,
            ResponseCode::NXDOMAIN => 3,
        }
    }
}

pub fn generate_response(
    id: u16,
    response_code: ResponseCode,
) -> Result<[u8; 512], Box<dyn std::error::Error>> {
    let flags = Flags {
        response_code: response_code.into(),
        query: false,
        ..Flags::default()
    };

    let header = Header {
        id,
        flags,
        ..Header::default()
    };

    let mut packet = BytesMut::with_capacity(512);
    let h: [u8; 12] = header.into();
    packet.extend_from_slice(h.as_slice());
    packet.put_bytes(0, 500);
    Ok(packet.to_vec().try_into().unwrap())
}

#[derive(Debug)]
pub struct DnsParser {
    pub buf: [u8; 512],
    position: usize,
}

impl DnsParser {
    pub fn new(buf: [u8; 512]) -> Self {
        Self { buf, position: 0 }
    }

    fn take_bytes(&mut self, n: usize) -> usize {
        let out = self.peek_bytes(n);
        self.position += n;
        out
    }

    fn peek_bytes(&self, n: usize) -> usize {
        self.buf[self.position..]
            .iter()
            .take(n)
            .fold(0usize, |acc, byte| acc << 8 | *byte as usize)
    }

    fn take(&mut self, n: usize) -> Vec<u8> {
        // TODO: this does not have to allocate, return slice
        let out = self.buf[self.position..].iter().take(n).cloned().collect();
        self.position += n;
        out
    }

    fn get<const N: usize>(&self) -> [u8; N] {
        // TODO: this does not have to allocate, return slice instead of array
        self.buf[self.position..]
            .iter()
            .take(N)
            .cloned()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    fn parse_domain_name(&mut self) -> String {
        // parse query (again)
        // https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
        // https://github.com/EmilHernvall/dnsguide/blob/master/chapter1.md
        let mut name = String::new();
        self.parse_domain_name_rec(&mut name);
        name
    }

    fn parse_domain_name_rec(&mut self, buf: &mut String) {
        // parse query (again)
        // https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
        // https://github.com/EmilHernvall/dnsguide/blob/master/chapter1.md
        if self.peek_bytes(1).eq(&0xC0) {
            let offset = self.take_bytes(2) ^ 0xC000;
            let old_position = self.position;
            self.position = offset;
            self.parse_domain_name_rec(buf);
            self.position = old_position;
        } else {
            self.parse_domain_name_inline(buf);
        }
    }

    fn parse_domain_name_inline(&mut self, buf: &mut String) {
        let mut next = self.peek_bytes(1);
        if next.eq(&192) {
            return;
        }
        // TODO: look to do this in one operation
        while next > 0 && next.ne(&192) {
            self.take_bytes(1);
            for c in self.take(next) {
                buf.push(c as char);
            }
            next = self.peek_bytes(1);
            if next > 0 {
                buf.push('.');
            }
            if next.eq(&192) {
                self.parse_domain_name_rec(buf);
                return;
            }
        }
        // take 0 octet
        self.take_bytes(1);
    }

    pub fn parse_question(self: &mut DnsParser) -> Question {
        Question {
            domain_name: self.parse_domain_name(),
            r#type: self.take_bytes(2),
            class: self.take_bytes(2),
        }
    }

    pub fn parse_answer(&mut self) -> Answer {
        // parse resource record
        // https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.3
        let name = self.parse_domain_name();
        let record_type: RecordType = self.take_bytes(2).into();
        let class = self.take_bytes(2);
        let ttl = self.take_bytes(4);
        let len = self.take_bytes(2);

        let meta = AnswerMeta {
            name,
            class,
            len,
            ttl,
            r#type: record_type,
        };

        match record_type {
            RecordType::A => {
                let ipv4 = self.get::<4>();
                self.position += 4;
                Answer::A {
                    meta,
                    ipv4: ipv4.into(),
                }
            }
            RecordType::CNAME => {
                let cname = self.parse_domain_name();
                Answer::CNAME { cname, meta }
            }
            RecordType::NS => todo!(),
            RecordType::MD => todo!(),
            RecordType::MF => todo!(),
            // todo
            RecordType::SOA => todo!(),
            RecordType::MB => todo!(),
            RecordType::MG => todo!(),
            RecordType::MR => todo!(),
            RecordType::NULL => todo!(),
            RecordType::WKS => todo!(),
            RecordType::PTR => todo!(),
            RecordType::HINFO => todo!(),
            RecordType::MINFO => todo!(),
            RecordType::MX => todo!(),
            RecordType::TXT => todo!(),
            RecordType::AXFR => todo!(),
            RecordType::MAILB => todo!(),
            RecordType::MAILA => todo!(),
            RecordType::ANY => todo!(),
            RecordType::URI => todo!(),
            RecordType::OTHER => todo!(),
        }
    }

    pub fn parse_header(&mut self) -> Header {
        Header {
            id: self.take_bytes(2) as u16,
            flags: Flags::from(self.take_bytes(2) as u16),
            question_count: self.take_bytes(2) as u16,
            answer_count: self.take_bytes(2) as u16,
            authority_count: self.take_bytes(2) as u16,
            additional_count: self.take_bytes(2) as u16,
        }
    }

    pub fn parse_answers(
        mut self,
    ) -> Result<(Vec<Answer>, [u8; 512]), Box<dyn std::error::Error + Send + Sync>> {
        let header = self.parse_header();

        for _ in 0..header.question_count {
            self.parse_question();
        }

        let answers = (0..header.answer_count)
            .map(|_| self.parse_answer())
            .collect::<Vec<_>>();

        Ok((answers, self.buf))
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(clippy::upper_case_acronyms)]
enum RecordType {
    A,     // 1 a host address
    NS,    // 2 an authoritative name server
    MD,    // 3 a mail destination (Obsolete - use MX)
    MF,    // 4 a mail forwarder (Obsolete - use MX)
    CNAME, // 5 the canonical name for an alias
    SOA,   // 6 marks the start of a zone of authority
    MB,    // 7 a mailbox domain name (EXPERIMENTAL)
    MG,    // 8 a mail group member (EXPERIMENTAL)
    MR,    // 9 a mail rename domain name (EXPERIMENTAL)
    NULL,  // 10 a null RR (EXPERIMENTAL)
    WKS,   // 11 a well known service description
    PTR,   // 12 a domain name pointer
    HINFO, // 13 host information
    MINFO, // 14 mailbox or mail list information
    MX,    // 15 mail exchange
    TXT,   // 16 text strings1
    // QTYPEs
    AXFR,  // 252 A request for a transfer of an entire zone
    MAILB, // 253 A request for mailbox-related records (MB, MG or MR)
    MAILA, // 254 A request for mail agent RRs (Obsolete - see MX)
    ANY,   // 255 A request for all records
    URI,   // 256
    OTHER,
}

impl From<usize> for RecordType {
    fn from(input: usize) -> Self {
        match input {
            1 => Self::A,
            2 => Self::NS,
            3 => Self::MD,
            4 => Self::MF,
            5 => Self::CNAME,
            6 => Self::SOA,
            7 => Self::MB,
            8 => Self::MG,
            9 => Self::MR,
            10 => Self::NULL,
            11 => Self::WKS,
            12 => Self::PTR,
            13 => Self::HINFO,
            14 => Self::MINFO,
            15 => Self::MX,
            16 => Self::TXT,
            252 => Self::AXFR,
            253 => Self::MAILB,
            254 => Self::MAILA,
            255 => Self::ANY,
            256 => Self::URI,
            _ => Self::OTHER,
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Header {
    pub id: u16,
    pub flags: Flags,
    pub question_count: u16,
    pub answer_count: u16,
    pub authority_count: u16,
    pub additional_count: u16,
}

impl From<Header> for [u8; 12] {
    fn from(header: Header) -> Self {
        let mut value = vec![];
        value.extend_from_slice(&header.id.to_be_bytes());
        let raw_flags: u16 = header.flags.into();
        value.extend_from_slice(&raw_flags.to_be_bytes());
        value.extend_from_slice(&header.question_count.to_be_bytes());
        value.extend_from_slice(&header.answer_count.to_be_bytes());
        value.extend_from_slice(&header.authority_count.to_be_bytes());
        value.extend_from_slice(&header.additional_count.to_be_bytes());
        value.try_into().unwrap()
    }
}

#[derive(Debug)]
pub struct Question {
    pub domain_name: String,
    pub r#type: usize,
    pub class: usize,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct AnswerMeta {
    name: String,
    r#type: RecordType,
    class: usize,
    ttl: usize,
    len: usize,
}

#[derive(Debug)]
pub enum Answer {
    A { meta: AnswerMeta, ipv4: Ipv4Addr },
    CNAME { meta: AnswerMeta, cname: String },
}

pub(crate) fn encode_domain_name(domain_name: &str) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(domain_name.len());
    domain_name.split('.').for_each(|part| {
        encoded.push(part.len() as u8);
        encoded.extend(part.as_bytes());
    });
    encoded.push(0);
    encoded
}

#[cfg(test)]
mod tests {

    use bytes::{BufMut, BytesMut};

    use crate::dns::{encode_domain_name, DnsParser, Flags, Header};

    #[test]
    fn test_response_parser_take() {
        let mut input = [0u8; 512];
        input[0] = 0x3;
        input[1] = 0x2;
        input[2] = 0x1;
        let mut parser = DnsParser::new(input);
        assert_eq!(parser.take_bytes(3), (0x3 << 16) | (0x2 << 8) | 0x1);
        assert_eq!(parser.buf.len(), 512);
    }

    #[test]
    fn test_response_parser_get() {
        let mut input = [0u8; 512];
        input[0] = 0x3;
        input[1] = 0x2;
        input[2] = 0x1;
        let parser = DnsParser::new(input);
        assert_eq!(parser.get::<3>(), [0x3, 0x2, 0x1]);
        assert_eq!(parser.buf.len(), 512);
    }

    #[test]
    fn test_conversion_flags() {
        let raw = 0x8100_u16; // response & recursive resolution desired flags set
        let flags = Flags::from(raw);
        assert_eq!(
            flags,
            Flags {
                query: false,
                recursion_desired: true,
                ..Default::default()
            }
        );

        let encoded: u16 = flags.into();
        assert_eq!(raw, encoded);
    }

    #[test]
    fn test_conversion_header() {
        let header = Header {
            flags: Flags::from(0x8100_u16),
            question_count: 1,
            answer_count: 1,
            id: 1234,
            ..Default::default()
        };
        let mut packet = BytesMut::with_capacity(512);
        let h: [u8; 12] = header.clone().into();
        packet.extend_from_slice(h.as_slice());
        packet.put_bytes(0, 500);
        let mut parser = DnsParser::new(packet.to_vec().try_into().unwrap());
        let deserialized_header = parser.parse_header();
        assert_eq!(header, deserialized_header);
    }

    #[test]
    fn test_encode_domain_name() {
        let res = encode_domain_name("www.example.com");
        assert_eq!(
            res,
            vec![
                // www.example.com
                3, 0x77, 0x77, 0x77, 7, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 3, 0x63, 0x6f,
                0x6d, 0
            ]
        );
    }
}

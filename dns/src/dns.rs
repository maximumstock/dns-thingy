use std::net::Ipv4Addr;

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Flags {
    query: bool,
    opcode: u8,
    authoritative_answer: bool,
    truncation: bool,
    recursion_desired: bool,
    recursion_available: bool,
    z: u8,
    response_code: u8,
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
        value |= (if flags.query { 0 } else { 1 }) << 15;
        value |= (flags.opcode as u16) << 11;
        value |= (if flags.authoritative_answer { 1 } else { 0 }) << 10;
        value |= (if flags.truncation { 1 } else { 0 }) << 9;
        value |= (if flags.recursion_desired { 1 } else { 0 }) << 8;
        value |= (if flags.recursion_available { 1 } else { 0 }) << 7;
        value |= (flags.z as u16) << 3;
        value |= flags.response_code as u16;
        value
    }
}

#[derive(Debug)]
pub(crate) struct ResponseParser {
    buf: Vec<u8>,
    position: usize,
}

impl ResponseParser {
    pub(crate) fn new(buf: Vec<u8>) -> Self {
        Self { buf, position: 0 }
    }

    pub(crate) fn is_done(&self) -> bool {
        self.buf.len() == self.position
    }

    fn take_bytes(&mut self, n: usize) -> usize {
        let out = self.peek_bytes(n);
        self.position += n;
        out
    }

    fn peek_bytes(&self, n: usize) -> usize {
        let out = self.buf[self.position..]
            .iter()
            .take(n)
            .fold(0usize, |acc, byte| acc << 8 | *byte as usize);
        out
    }

    fn take(&mut self, n: usize) -> Vec<u8> {
        let out = self.buf[self.position..].iter().take(n).cloned().collect();
        self.position += n;
        out
    }

    fn get<const N: usize>(&self) -> [u8; N] {
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

    pub(crate) fn parse_question(self: &mut ResponseParser) -> Question {
        Question {
            domain_name: self.parse_domain_name(),
            r#type: self.take_bytes(2),
            class: self.take_bytes(2),
        }
    }

    pub(crate) fn parse_answer(&mut self) -> Answer {
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
        }
    }

    pub(crate) fn parse_header(&mut self) -> Header {
        Header {
            id: self.take_bytes(2),
            flags: Flags::from(self.take_bytes(2) as u16),
            question_count: self.take_bytes(2),
            answer_count: self.take_bytes(2),
            authority_count: self.take_bytes(2),
            additional_count: self.take_bytes(2),
        }
    }
}

#[derive(Debug, Copy, Clone)]
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
            _ => {
                unreachable!();
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Header {
    pub(crate) id: usize,
    pub(crate) flags: Flags,
    pub(crate) question_count: usize,
    pub(crate) answer_count: usize,
    pub(crate) authority_count: usize,
    pub(crate) additional_count: usize,
}

#[derive(Debug)]
pub(crate) struct Question {
    domain_name: String,
    r#type: usize,
    class: usize,
}

#[derive(Debug)]
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
    let mut encoded = Vec::with_capacity(domain_name.len() * 2);
    domain_name.split('.').for_each(|part| {
        encoded.push(part.len() as u8);
        encoded.extend(part.as_bytes());
    });
    encoded.push(0);
    encoded
}

#[cfg(test)]
mod tests {

    use crate::dns::{encode_domain_name, Flags, ResponseParser};

    #[test]
    fn test_response_parser_take() {
        let mut parser = ResponseParser::new(vec![0x3, 0x2, 0x1]);
        assert_eq!(parser.take_bytes(3), (0x3 << 16) | (0x2 << 8) | 0x1);
        assert_eq!(parser.buf.len(), 3);
    }

    #[test]
    fn test_response_parser_get() {
        let parser = ResponseParser::new(vec![0x3, 0x2, 0x1]);
        assert_eq!(parser.get::<3>(), [0x3, 0x2, 0x1]);
        assert_eq!(parser.buf.len(), 3);
    }

    #[test]
    fn test_parse_flags() {
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
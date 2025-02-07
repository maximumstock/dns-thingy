use std::time::Duration;

use crate::protocol::{
    answer::{Answer, AnswerMeta, AnswerValue},
    header::{Flags, Header},
    question::Question,
    record_type::RecordType,
};

pub type DnsPacketBuffer = [u8; 512];

#[derive(Debug)]
pub struct DnsParser<'a> {
    pub buf: &'a DnsPacketBuffer,
    position: usize,
    /// Internal metadata about the buffer indices of the answer TTL fields live
    answer_ttl_indices: Vec<usize>,
}

pub trait Collate {
    fn collate(self) -> usize;
}

impl<'a> Collate for &'a [u8] {
    /// Folds `&[u8; N]` into a single usize, so it only makes sense for `N` <= 8
    fn collate(self: &'a [u8]) -> usize {
        self.iter()
            .fold(0usize, |acc, byte| acc << 8 | *byte as usize)
    }
}

impl<const N: usize> Collate for [u8; N] {
    /// Folds `[u8; N]` into a single usize, so it only makes sense for `N` <= 8
    fn collate(self: [u8; N]) -> usize {
        self.iter()
            .fold(0usize, |acc, byte| acc << 8 | *byte as usize)
    }
}

impl<'a> DnsParser<'a> {
    pub fn new(buf: &'a DnsPacketBuffer) -> Self {
        Self {
            buf,
            position: 0,
            answer_ttl_indices: vec![],
        }
    }

    fn peek(&self, n: usize) -> &[u8] {
        &self.buf[self.position..self.position + n]
    }

    fn advance(&mut self, n: usize) -> &[u8] {
        let out = &self.buf[self.position..self.position + n];
        self.position += n;
        out
    }

    fn advance_n<const N: usize>(&mut self) -> [u8; N] {
        let out = self.buf[self.position..self.position + N]
            .try_into()
            .unwrap();
        self.position += N;
        out
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
        if self.peek(1).collate().eq(&0xC0) {
            let offset = self.advance_n::<2>().collate() ^ 0xC000;
            let old_position = self.position;
            self.position = offset;
            self.parse_domain_name_rec(buf);
            self.position = old_position;
        } else {
            self.parse_domain_name_inline(buf);
        }
    }

    fn parse_domain_name_inline(&mut self, buf: &mut String) {
        let mut next = self.peek(1).collate();
        if next.eq(&192) {
            return;
        }
        // TODO: look to do this in one operation
        while next > 0 && next.ne(&192) {
            self.advance_n::<1>().collate();
            for c in self.advance(next) {
                buf.push(*c as char);
            }
            next = self.peek(1).collate();
            if next > 0 {
                buf.push('.');
            }
            if next.eq(&192) {
                self.parse_domain_name_rec(buf);
                return;
            }
        }
        // skip 0 byte at the end
        self.position += 1;
    }

    // Question section format https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.2
    pub fn parse_question(&mut self) -> Question {
        Question {
            domain_name: self.parse_domain_name(),
            r#type: RecordType::from(self.advance_n::<2>().collate() as u16),
            class: self.advance_n::<2>().collate() as u16,
        }
    }

    // Resource section format https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.3
    pub fn parse_answer(&mut self) -> Answer {
        // parse resource record
        // https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.3
        let name = self.parse_domain_name();
        let record_type: RecordType = (self.advance_n::<2>().collate() as u16).into();
        let class = self.advance_n::<2>().collate() as u16;
        self.answer_ttl_indices.push(self.position);
        let ttl = self.advance_n::<4>().collate() as u32;
        let len = self.advance_n::<2>().collate() as u16;

        let meta = AnswerMeta {
            name,
            class,
            len,
            ttl,
            r#type: record_type,
        };

        // See Section 3.3 Standard RRs (https://datatracker.ietf.org/doc/html/rfc1035#section-3.3) for an overview
        // of how to parse certain record types
        // More record types to parse at some point: https://en.wikipedia.org/wiki/List_of_DNS_record_types
        let answer_value = match record_type {
            // CNAME https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.1
            RecordType::CNAME => {
                let cname = self.parse_domain_name();
                AnswerValue::CNAME { cname }
            }
            // HINFO https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.2
            RecordType::HINFO => todo!(),
            // MB https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.3
            RecordType::MB => todo!(),
            // MD https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.4
            RecordType::MD => todo!(),
            // MF https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.5
            RecordType::MF => todo!(),
            // MG https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.6
            RecordType::MG => todo!(),
            // MINFO https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.7
            RecordType::MINFO => todo!(),
            // MR https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.8
            RecordType::MR => todo!(),
            // MX https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.9
            RecordType::MX => {
                let preference = self.advance_n::<2>().collate() as u16;
                let exchange = self.parse_domain_name();
                AnswerValue::MX {
                    preference,
                    exchange,
                }
            }
            // NULL https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.10
            RecordType::NULL => todo!(),
            // NS https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.11
            RecordType::NS => {
                let ns = self.parse_domain_name();
                AnswerValue::NS { ns }
            }
            // PTR https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.12
            RecordType::PTR => {
                let domain_name = self.parse_domain_name();
                AnswerValue::PTR { domain_name }
            }
            // SOA https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.13
            RecordType::SOA => {
                let mname = self.parse_domain_name();
                let rname = self.parse_domain_name();
                let serial = self.advance_n::<4>().collate() as u32;
                let refresh = self.advance_n::<4>().collate() as u32;
                let retry = self.advance_n::<4>().collate() as u32;
                let expire = self.advance_n::<4>().collate() as u32;
                let minimum = self.advance_n::<4>().collate() as u32;

                AnswerValue::SOA {
                    mname,
                    rname,
                    serial,
                    refresh,
                    retry,
                    expire,
                    minimum,
                }
            }
            // TXT https://datatracker.ietf.org/doc/html/rfc1035#section-3.3.14
            RecordType::TXT => todo!(),
            // A https://datatracker.ietf.org/doc/html/rfc1035#section-3.4.1
            RecordType::A => {
                let ipv4 = self.advance_n::<4>();
                AnswerValue::A { ipv4: ipv4.into() }
            }
            // AAAA https://datatracker.ietf.org/doc/html/rfc3596#section-2.2
            RecordType::AAAA => {
                let ipv6 = self.advance_n::<16>();
                AnswerValue::AAAA { ipv6: ipv6.into() }
            }
            RecordType::OTHER(_) => todo!(),
        };

        Answer::new(meta, answer_value)
    }

    // Header section format https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.1
    pub fn parse_header(&mut self) -> Header {
        Header {
            request_id: self.advance_n::<2>().collate() as u16,
            flags: Flags::from(self.advance_n::<2>().collate() as u16),
            question_count: self.advance_n::<2>().collate() as u16,
            answer_count: self.advance_n::<2>().collate() as u16,
            authority_count: self.advance_n::<2>().collate() as u16,
            additional_count: self.advance_n::<2>().collate() as u16,
        }
    }

    pub fn parse(&mut self) -> Result<DnsPacket, Box<dyn std::error::Error + Send + Sync>> {
        let header = self.parse_header();

        // We only pick the first question, since multiple questions seem to be unsupported by most
        // nameservers anyways, see https://stackoverflow.com/questions/4082081/requesting-a-and-aaaa-records-in-single-dns-query/4083071#4083071.
        let mut first_question = None;
        for _ in 0..header.question_count {
            if first_question.is_none() {
                first_question = Some(self.parse_question());
            }
        }

        // TODO: If we want to parse answers, we actually need to implement all QTYPEs, otherwise we fail at one
        // of the various `todo()`s downstream
        let answers = (0..header.answer_count)
            .map(|_| self.parse_answer())
            .collect::<Vec<_>>();

        Ok(DnsPacket {
            header,
            question: first_question.unwrap(),
            answers,
        })
    }

    /// This is a hack to take an existing `DnsPacketBuffer` server response and alter it so it can be re-used
    /// as a response to a later, identical DNS question.
    /// In order to do so, we need to make sure the DNS answer TTL values are decreased accordingly and the
    /// header request id has to be overwriten by the new DNS question's request id.
    pub fn update_cached_packet(
        mut self,
        ttl_reduction: Duration,
        new_request_id: u16,
    ) -> Result<[u8; 512], Box<dyn std::error::Error + Send + Sync>> {
        self.position = 0;

        let seconds = ttl_reduction.as_secs() as u32; // Each entry has a u32 TTL
        let mut buf_copy = *self.buf;

        self.parse()?;

        println!("[Cache] Reducing ttl by {}s", seconds);

        // Update the TTL values
        for &start_index in &self.answer_ttl_indices {
            let old_ttl = buf_copy[start_index..start_index + 4].collate() as u32;
            let new_ttl = old_ttl - seconds;
            buf_copy[start_index..start_index + 4]
                .copy_from_slice(new_ttl.to_be_bytes().as_slice());
        }

        // Update the request id to match
        buf_copy[0..2].copy_from_slice(new_request_id.to_be_bytes().as_slice());

        Ok(buf_copy)
    }
}

#[derive(Clone, Debug)]
pub struct DnsPacket {
    pub header: Header,
    pub question: Question,
    pub answers: Vec<Answer>,
}

// TODO: Compared to the actual domain name encoding schema, this is very inefficient and might break
// with many long record names, since we don't reduce subdomains. Either we need that or we
// should just never encode this ourselves and skip serialization altogether, since we only need
// two write `DnsPacket` when alterting TTL values, which we can without any issues by just jumping
// to the relevant byte elements via the parser.
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
    use crate::{
        parse::parser::{encode_domain_name, Collate, DnsParser},
        protocol::header::{Flags, Header},
    };

    #[test]
    fn test_parser_advance() {
        let mut input = [0u8; 512];
        input[0..3].copy_from_slice(&[0x3, 0x2, 0x1]);

        let mut parser = DnsParser::new(&input);
        assert_eq!(
            parser.advance_n::<3>().collate(),
            (0x3 << 16) | (0x2 << 8) | 0x1
        );
        assert_eq!(parser.buf.len(), 512);
    }

    #[test]
    fn test_parser_peek_n() {
        let mut input = [0u8; 512];
        input[0..3].copy_from_slice(&[0x3, 0x2, 0x1]);

        let parser = DnsParser::new(&input);
        assert_eq!(parser.peek(3), [0x3, 0x2, 0x1]);
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
            request_id: 1234,
            ..Default::default()
        };

        let mut packet = [0u8; 512];
        let serialized_header: [u8; 12] = header.clone().into();
        packet[0..12].copy_from_slice(&serialized_header);

        let mut parser = DnsParser::new(&packet);
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

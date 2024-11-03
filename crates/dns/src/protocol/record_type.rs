#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum RecordType {
    // TYPEs, see https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2
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
    // WKS,   // 11 a well known service description
    PTR,   // 12 a domain name pointer
    HINFO, // 13 host information
    MINFO, // 14 mailbox or mail list information
    MX,    // 15 mail exchange
    TXT,   // 16 text strings1
    AAAA,  // IPv6 extension, see https://datatracker.ietf.org/doc/html/rfc3596#section-2.1
    // HTTPS, // HTTPS & SVCB extension, see https://datatracker.ietf.org/doc/rfc9460/
    // QTYPEs, see https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.3
    // AXFR,  // 252 A request for a transfer of an entire zone
    // MAILB, // 253 A request for mailbox-related records (MB, MG or MR)
    // MAILA, // 254 A request for mail agent RRs (Obsolete - see MX)
    // ANY,   // 255 A request for all records
    // URI,   // 256
    OTHER(usize),
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
            // 11 => Self::WKS,
            12 => Self::PTR,
            13 => Self::HINFO,
            14 => Self::MINFO,
            15 => Self::MX,
            16 => Self::TXT,
            28 => Self::AAAA,
            // 65 => Self::HTTPS,
            // 252 => Self::AXFR,
            // 253 => Self::MAILB,
            // 254 => Self::MAILA,
            // 255 => Self::ANY,
            // 256 => Self::URI,
            a => Self::OTHER(a),
        }
    }
}

impl From<RecordType> for usize {
    fn from(value: RecordType) -> Self {
        match value {
            RecordType::A => 1,
            RecordType::NS => 2,
            RecordType::MD => 3,
            RecordType::MF => 4,
            RecordType::CNAME => 5,
            RecordType::SOA => 6,
            RecordType::MB => 7,
            RecordType::MG => 8,
            RecordType::MR => 9,
            RecordType::NULL => 10,
            // RecordType::WKS => 11,
            RecordType::PTR => 12,
            RecordType::HINFO => 13,
            RecordType::MINFO => 14,
            RecordType::MX => 15,
            RecordType::TXT => 16,
            RecordType::AAAA => 28,
            // RecordType::HTTPS => 65,
            // RecordType::AXFR => 252,
            // RecordType::MAILB => 253,
            // RecordType::MAILA => 254,
            // RecordType::ANY => 255,
            // RecordType::URI => 256,
            RecordType::OTHER(v) => v,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
/// This enum models all possibly occurring record types in DNS resource records.
///
/// The record type determines how the `RDATA` field in the respective resource record
/// has to be parsed. Other than that `RDATA` field, all other resource record data occurrs
/// in a shared format.
///
/// You can read in the RFCs that DNS question resource records can use `TYPE` and `QTYPE`
/// record types whereas other resource records, such as answer, authorities and additional
/// resource records, cannot use `QTYPE` as their record type. Since we model all record types
/// within one enum here, this means that we are not properly modeling the world, as answer
/// resource records cannot occurr with for example a record type of `AXFR`. We can get away
/// with this as we mostly don't care about the `RDATA` record type specific data and instead
/// can focus on parsing the shared header.
///
/// For some record types we still parse `RDATA`, simply because it makes the model a bit more
/// useful, e.g. parsing the IPv4 address for `A` record type resource records.
///
/// TODO: Update this list based on https://en.wikipedia.org/wiki/List_of_DNS_record_types
pub enum RecordType {
    // RFC 1035 defines
    // - 16 TYPEs, see https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2
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
    // - 4 QTYPEs, see https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.3
    AXFR,  // 252 A request for a transfer of an entire zone
    MAILB, // 253 A request for mailbox-related records (MB, MG or MR)
    MAILA, // 254 A request for mail agent RRs (Obsolete - see MX)
    ANY,   // 255 A request for all records
    // Pseudo RR type - see https://en.wikipedia.org/wiki/List_of_DNS_record_types
    OPT, // 41 A pseudo record type to support EDNS.
    // Later Extensions
    AAAA, // IPv6 host address, see RFC 3596 defines extra types https://datatracker.ietf.org/doc/html/rfc3596#section-2.1
    HTTPS, // HTTPS & SVCB extension, see RFC 9460 https://datatracker.ietf.org/doc/rfc9460/
    // Fallback
    Unknown(u16),
}

impl From<u16> for RecordType {
    fn from(input: u16) -> Self {
        match input {
            // TYPE
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
            // QTYPE
            252 => Self::AXFR,
            253 => Self::MAILB,
            254 => Self::MAILA,
            255 => Self::ANY,
            // Other
            41 => Self::OPT,
            // Extensions
            28 => Self::AAAA,
            65 => Self::HTTPS,
            _ => Self::Unknown(input),
        }
    }
}

impl From<RecordType> for u16 {
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
            RecordType::WKS => 11,
            RecordType::PTR => 12,
            RecordType::HINFO => 13,
            RecordType::MINFO => 14,
            RecordType::MX => 15,
            RecordType::TXT => 16,
            RecordType::OPT => 41,
            RecordType::AAAA => 28,
            RecordType::HTTPS => 65,
            RecordType::AXFR => 252,
            RecordType::MAILB => 253,
            RecordType::MAILA => 254,
            RecordType::ANY => 255,
            RecordType::Unknown(n) => n,
        }
    }
}

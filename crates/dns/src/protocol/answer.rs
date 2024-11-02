use std::net::{Ipv4Addr, Ipv6Addr};

use super::record_type::RecordType;

#[derive(Debug, Clone)]
pub struct AnswerMeta {
    pub name: String,
    pub r#type: RecordType,
    pub class: usize,
    pub ttl: usize,
    pub len: usize,
}

#[derive(Debug, Clone)]
pub enum Answer {
    A {
        meta: AnswerMeta,
        ipv4: Ipv4Addr,
    },
    AAAA {
        meta: AnswerMeta,
        ipv6: Ipv6Addr,
    },
    CNAME {
        meta: AnswerMeta,
        cname: String,
    },
    NS {
        ns: String,
        meta: AnswerMeta,
    },
    MB {
        domain_name: String,
        meta: AnswerMeta,
    },
    MX {
        preference: u16,
        exchange: String,
        meta: AnswerMeta,
    },
    PTR {
        domain_name: String,
        meta: AnswerMeta,
    },
    SOA {
        meta: AnswerMeta,
        mname: String,
        rname: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    },
}

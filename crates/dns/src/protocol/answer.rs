use std::net::{Ipv4Addr, Ipv6Addr};

use super::record_type::RecordType;

#[derive(Debug, Clone)]
pub struct AnswerMeta {
    pub name: String,
    pub r#type: RecordType,
    pub class: usize,
    pub ttl: u32,
    pub len: usize,
}

#[derive(Debug, Clone)]
pub struct Answer {
    pub meta: AnswerMeta,
    pub value: AnswerValue,
}

impl Answer {
    pub fn new(meta: AnswerMeta, value: AnswerValue) -> Self {
        Self { meta, value }
    }
}

#[derive(Debug, Clone)]
pub enum AnswerValue {
    A {
        ipv4: Ipv4Addr,
    },
    AAAA {
        ipv6: Ipv6Addr,
    },
    CNAME {
        cname: String,
    },
    NS {
        ns: String,
    },
    MB {
        domain_name: String,
    },
    MX {
        preference: u16,
        exchange: String,
    },
    PTR {
        domain_name: String,
    },
    SOA {
        mname: String,
        rname: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    },
}

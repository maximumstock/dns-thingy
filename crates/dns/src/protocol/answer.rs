use std::net::{Ipv4Addr, Ipv6Addr};

use super::record_type::RecordType;

#[derive(Debug, Clone)]
pub struct ResourceRecordMeta {
    pub name: String,
    pub record_type: RecordType,
    pub class: u16,
    pub ttl: u32,
    pub len: u16,
}

#[derive(Debug, Clone)]
pub struct ResourceRecord {
    pub meta: ResourceRecordMeta,
    pub value: ResourceRecordData,
}

impl ResourceRecord {
    pub fn new(meta: ResourceRecordMeta, value: ResourceRecordData) -> Self {
        Self { meta, value }
    }
}

#[derive(Debug, Clone)]
pub enum ResourceRecordData {
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
    /// Marks the RR data for a record type for which we haven't implemented the parsing step yet.
    /// We can afford this since we often don't care about RR data and only about the RR metadata.
    Unknown,
}

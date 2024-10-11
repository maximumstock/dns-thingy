use std::net::Ipv4Addr;

use super::record_type::RecordType;

#[derive(Debug)]
pub struct AnswerMeta {
    pub name: String,
    pub r#type: RecordType,
    pub class: usize,
    pub ttl: usize,
    pub len: usize,
}

#[derive(Debug)]
pub enum Answer {
    A { meta: AnswerMeta, ipv4: Ipv4Addr },
    CNAME { meta: AnswerMeta, cname: String },
}

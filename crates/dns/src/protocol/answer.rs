use std::net::Ipv4Addr;

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
    A { meta: AnswerMeta, ipv4: Ipv4Addr },
    CNAME { meta: AnswerMeta, cname: String },
}

impl Answer {
    pub fn get_domain_name(&self) -> &str {
        match self {
            Answer::A { meta, ipv4: _ } => &meta.name,
            Answer::CNAME { meta, cname: _ } => &meta.name,
        }
    }

    pub fn get_record_type(&self) -> RecordType {
        match self {
            Answer::A { meta, ipv4: _ } => meta.r#type,
            Answer::CNAME { meta, cname: _ } => meta.r#type,
        }
    }
}

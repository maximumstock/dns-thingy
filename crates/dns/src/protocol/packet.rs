use crate::protocol::{answer::ResourceRecord, header::Header, question::Question};

#[derive(Clone, Debug)]
pub struct DnsPacket {
    pub header: Header,
    pub question: Question,
    /// The list of resource records that describe the upstream DNS answers.
    pub answers: Vec<ResourceRecord>,
    /// The list of resource records that describe nameserver authorities.
    pub authorities: Vec<ResourceRecord>,
    /// The list of resource records that upstream sent as additional data.
    pub additional: Vec<ResourceRecord>,
}

use super::record_type::RecordType;

#[derive(Debug, Clone)]
pub struct Question {
    pub domain_name: String,
    pub r#type: RecordType,
    pub class: usize,
}

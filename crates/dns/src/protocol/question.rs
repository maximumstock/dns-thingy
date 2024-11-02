#[derive(Debug, Clone)]
pub struct Question {
    pub domain_name: String,
    pub r#type: usize,
    pub class: usize,
}

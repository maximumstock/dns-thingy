/// This module houses all code related to creating and handling filter rules.

pub fn apply_domain_filter(domain: &str) -> bool {
    domain.eq("google.de")
}

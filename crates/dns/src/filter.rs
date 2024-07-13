/// This module houses all code related to creating and handling filter rules.

pub fn is_domain_blacklisted(domain: &str) -> bool {
    domain.eq("google.de")
}

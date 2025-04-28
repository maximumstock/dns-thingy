use std::collections::BTreeSet;

/// This module houses all code related to creating and handling filter rules.
pub fn is_domain_blacklisted(blocked_domains: &BTreeSet<String>, domain: &str) -> bool {
    blocked_domains.iter().any(|blocked| domain.eq(blocked))
}

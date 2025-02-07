use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    time::{Duration, Instant},
};

use dns::{
    parser::{DnsPacket, DnsPacketBuffer, DnsParser},
    protocol::record_type::RecordType,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct RequestCache {
    inner: HashMap<CacheKey, CacheValue>,
}

impl RequestCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&mut self, key: CacheKey, new_request_id: u16) -> Option<CacheValue> {
        match self.inner.entry(key) {
            Vacant(_) => None,
            Occupied(mut entry) => {
                let cached = entry.get_mut();
                if cached.is_valid() {
                    // todo: maybe we should not re-parse the packet on every cache hit
                    let parser = DnsParser::new(&cached.packet);
                    let updated_packet: DnsPacketBuffer = parser
                        .update_cached_packet(cached.get_remaining_ttl(), new_request_id)
                        .expect("Could not parse and reduce ttl of DNS packet answers");

                    // After constructing a new reply packet, we need to update the cache entry
                    cached.packet = updated_packet;
                    cached.cached_at = Instant::now();

                    Some(cached.clone())
                } else {
                    entry.remove_entry();
                    None
                }
            }
        }
    }

    pub fn set(&mut self, key: CacheKey, buffer: DnsPacketBuffer) {
        self.inner.insert(key, CacheValue::new(buffer));
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub(crate) struct CacheKey {
    record_type: RecordType,
    domain: String,
}

impl CacheKey {
    pub fn new(record_type: RecordType, domain: String) -> Self {
        CacheKey {
            record_type,
            domain,
        }
    }

    pub(crate) fn from_packet(packet: &DnsPacket) -> Self {
        CacheKey::new(packet.question.r#type, packet.question.domain_name.clone())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CacheValue {
    /// The point in time when the cached entry expires. We need this to quickly check if an entry can be discarded or not.
    pub(crate) expires_at: Instant,
    /// The point in time when the entry was cached. We need this to calculate how much TTL is left when updating the cached `DnsPacketBuffer` buffer.
    pub(crate) cached_at: Instant,
    /// We cache the entire original buffer
    pub(crate) packet: DnsPacketBuffer,
}

impl CacheValue {
    pub fn new(reply: DnsPacketBuffer) -> Self {
        // We use the minimum TTL over all records in the DNS answer to calculate until when
        // the cached entry should still be usable
        let parsed = DnsParser::new(&reply).parse().unwrap();
        let remaining_ttl = parsed.answers.iter().map(|a| a.meta.ttl).min().unwrap_or(0);
        let expires_at = Instant::now()
            .checked_add(Duration::from_secs(remaining_ttl as u64))
            .unwrap();

        CacheValue {
            expires_at,
            cached_at: Instant::now(),
            packet: reply,
        }
    }

    pub fn is_valid(&self) -> bool {
        Instant::now() <= self.expires_at
    }

    pub fn get_remaining_ttl(&self) -> Duration {
        Instant::now().duration_since(self.cached_at)
    }
}

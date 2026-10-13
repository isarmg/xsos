use crate::error::{Error, Result};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

const ENTRY_TTL: Duration = Duration::from_secs(60 * 60);
const IDENTIFIER_CAPACITY: usize = 8_192;
const PAIRING_WORK_CONCURRENCY: usize = 16;

/// Only identities verified against persisted pairing state enter these maps.
/// Anonymous requests share a bounded work pool; pending rows have separate DB limits.
#[derive(Clone)]
pub(crate) struct PairingAdmission {
    state: Arc<Mutex<AdmissionState>>,
    work: Arc<Semaphore>,
}
impl PairingAdmission {
    pub(crate) fn production() -> Self {
        Self::new(
            BucketPolicy::new(30, Duration::from_secs(1)),
            BucketPolicy::new(6, Duration::from_secs(30)),
            IDENTIFIER_CAPACITY,
            ENTRY_TTL,
        )
    }
    fn new(poll: BucketPolicy, activation: BucketPolicy, capacity: usize, ttl: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(AdmissionState {
                poll_requests: BoundedBuckets::new(poll, capacity, ttl),
                activation_instances: BoundedBuckets::new(activation, capacity, ttl),
            })),
            work: Arc::new(Semaphore::new(PAIRING_WORK_CONCURRENCY)),
        }
    }
    #[cfg(test)]
    pub(crate) fn for_test(burst: u32, capacity: usize, ttl: Duration) -> Self {
        let policy = BucketPolicy::new(burst, Duration::from_secs(60));
        Self::new(policy, policy, capacity, ttl)
    }
    pub(crate) fn acquire_work(&self) -> Result<OwnedSemaphorePermit> {
        self.work
            .clone()
            .try_acquire_owned()
            .map_err(|_| Error::RateLimited {
                message: "pairing work capacity is busy",
                retry_after: 1,
            })
    }
    pub(crate) fn check_poll(&self, request_id: Uuid) -> Result<()> {
        self.lock()
            .poll_requests
            .check_at(request_id, Instant::now())
            .map_err(|delay| limited("pairing request poll rate exceeded", delay))
    }
    pub(crate) fn check_activation(&self, instance_id: Uuid) -> Result<()> {
        self.lock()
            .activation_instances
            .check_at(instance_id, Instant::now())
            .map_err(|delay| limited("pairing instance activation attempts exceeded", delay))
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, AdmissionState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
struct AdmissionState {
    poll_requests: BoundedBuckets<Uuid>,
    activation_instances: BoundedBuckets<Uuid>,
}
#[derive(Clone, Copy)]
struct BucketPolicy {
    burst: u32,
    refill_interval: Duration,
}

impl BucketPolicy {
    fn new(burst: u32, refill_interval: Duration) -> Self {
        assert!(burst > 0);
        assert!(!refill_interval.is_zero());
        Self {
            burst,
            refill_interval,
        }
    }
}

#[derive(Clone, Copy)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
    last_seen: Instant,
}

impl Bucket {
    fn new(policy: BucketPolicy, now: Instant) -> Self {
        Self {
            tokens: f64::from(policy.burst),
            last_refill: now,
            last_seen: now,
        }
    }

    fn tokens_at(self, policy: BucketPolicy, now: Instant) -> f64 {
        let elapsed = now.saturating_duration_since(self.last_refill);
        (self.tokens + elapsed.as_secs_f64() / policy.refill_interval.as_secs_f64())
            .min(f64::from(policy.burst))
    }

    fn check(&mut self, policy: BucketPolicy, now: Instant) -> std::result::Result<(), Duration> {
        self.tokens = self.tokens_at(policy, now);
        self.last_refill = now;
        self.last_seen = now;
        if self.tokens < 1.0 {
            let missing = 1.0 - self.tokens;
            return Err(Duration::from_secs_f64(
                missing * policy.refill_interval.as_secs_f64(),
            ));
        }
        self.tokens -= 1.0;
        Ok(())
    }
}

struct BoundedBuckets<K> {
    entries: HashMap<K, Bucket>,
    policy: BucketPolicy,
    capacity: usize,
    entry_ttl: Duration,
}

impl<K> BoundedBuckets<K>
where
    K: Clone + Eq + Hash,
{
    fn new(policy: BucketPolicy, capacity: usize, entry_ttl: Duration) -> Self {
        assert!(capacity > 0);
        assert!(!entry_ttl.is_zero());
        Self {
            entries: HashMap::new(),
            policy,
            capacity,
            entry_ttl,
        }
    }

    fn check_at(&mut self, key: K, now: Instant) -> std::result::Result<(), Duration> {
        self.entries
            .retain(|_, entry| now.saturating_duration_since(entry.last_seen) < self.entry_ttl);
        if !self.entries.contains_key(&key) {
            self.make_room(now)?;
            self.entries
                .insert(key.clone(), Bucket::new(self.policy, now));
        }
        self.entries
            .get_mut(&key)
            .expect("the bucket exists")
            .check(self.policy, now)
    }

    fn make_room(&mut self, now: Instant) -> std::result::Result<(), Duration> {
        if self.entries.len() < self.capacity {
            return Ok(());
        }
        let evictable = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.tokens_at(self.policy, now) >= f64::from(self.policy.burst))
            .min_by_key(|(_, entry)| entry.last_seen)
            .map(|(key, _)| key.clone());
        if let Some(key) = evictable {
            self.entries.remove(&key);
            return Ok(());
        }
        let retry = self
            .entries
            .values()
            .map(|entry| {
                let missing = f64::from(self.policy.burst) - entry.tokens_at(self.policy, now);
                Duration::from_secs_f64(
                    missing.max(0.0) * self.policy.refill_interval.as_secs_f64(),
                )
            })
            .min()
            .unwrap_or(self.entry_ttl);
        Err(retry.max(Duration::from_nanos(1)))
    }
}

fn limited(message: &'static str, delay: Duration) -> Error {
    Error::RateLimited {
        message,
        retry_after: delay
            .as_secs()
            .saturating_add(u64::from(delay.subsec_nanos() != 0))
            .max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_instance_and_request_buckets_are_independent_and_bounded() {
        let admission = PairingAdmission::for_test(1, 1, Duration::from_secs(300));
        let instance = Uuid::new_v4();
        admission.check_activation(instance).unwrap();
        assert!(matches!(
            admission.check_activation(instance),
            Err(Error::RateLimited { .. })
        ));
        admission.check_poll(Uuid::new_v4()).unwrap();
        assert!(matches!(
            admission.check_poll(Uuid::new_v4()),
            Err(Error::RateLimited {
                retry_after: 60,
                ..
            })
        ));
    }
    #[test]
    fn anonymous_work_is_globally_bounded_without_allocating_identity_buckets() {
        let admission = PairingAdmission::production();
        let permits: Vec<_> = (0..PAIRING_WORK_CONCURRENCY)
            .map(|_| admission.acquire_work().unwrap())
            .collect();
        assert!(admission.acquire_work().is_err());
        assert!(admission.lock().poll_requests.entries.is_empty());
        assert!(admission.lock().activation_instances.entries.is_empty());
        drop(permits);
        assert!(admission.acquire_work().is_ok());
    }
}

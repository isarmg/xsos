//! Current retention bounds and defaults, shared with contract generation.
use std::time::Duration;

pub const MIN_RAW_RETENTION: Duration = Duration::from_secs(24 * 60 * 60);
pub const MAX_RAW_RETENTION: Duration = Duration::from_secs(365 * 24 * 60 * 60);
pub const MAX_AGGREGATE_RETENTION: Duration = Duration::from_secs(3_650 * 24 * 60 * 60);
pub const MIN_MAINTENANCE_INTERVAL: Duration = Duration::from_secs(1);
pub const MAX_MAINTENANCE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
pub const MAX_RETENTION_BATCH_SIZE: usize = 512;
pub const MIN_RETENTION_TRANSACTIONS: usize = 3;
pub const MIN_RETENTION_RUN_TIME: Duration = Duration::from_millis(100);
pub const MAX_RETENTION_TRANSACTIONS: usize = 30;
pub const MAX_RETENTION_RUN_TIME: Duration = Duration::from_secs(10);
pub const MAX_RETENTION_YIELD: Duration = Duration::from_millis(100);
pub const DEFAULT_RAW_RETENTION_DAYS: u64 = 7;
pub const DEFAULT_AGGREGATE_RETENTION_DAYS: u64 = 365;
pub const DEFAULT_MAINTENANCE_INTERVAL_SECONDS: u64 = 300;
pub const DEFAULT_RETENTION_BATCH_SIZE: usize = 256;
pub const DEFAULT_RETENTION_TRANSACTIONS: usize = 12;
pub const DEFAULT_RETENTION_RUN_MILLISECONDS: u64 = 2_000;
pub const DEFAULT_RETENTION_YIELD_MILLISECONDS: u64 = 10;

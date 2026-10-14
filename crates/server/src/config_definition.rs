//! Current configuration data shared by runtime parsing and build-time contract generation.
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use crate::retention_defaults::*;

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub(crate) data_dir: PathBuf,
    pub(crate) bind: SocketAddr,
    pub(crate) development: bool,
    pub(crate) log_level: String,
    pub(crate) development_web_dir: Option<String>,
    pub(crate) telemetry_queue_capacity: usize,
    pub(crate) telemetry_batch_size: usize,
    pub(crate) telemetry_flush_milliseconds: u64,
    pub(crate) telemetry_enqueue_wait_milliseconds: u64,
    pub(crate) telemetry_request_timeout_milliseconds: u64,
    pub(crate) telemetry_shutdown_drain_milliseconds: u64,
    pub(crate) raw_retention_days: u64,
    pub(crate) aggregate_retention_days: u64,
    pub(crate) retention_interval_seconds: u64,
    pub(crate) retention_batch_size: usize,
    pub(crate) retention_max_transactions_per_run: usize,
    pub(crate) retention_max_run_milliseconds: u64,
    pub(crate) retention_yield_milliseconds: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("/opt/isarmg/xsos/state"),
            bind: "127.0.0.1:18105".parse().expect("valid default bind"),
            development: false,
            log_level: "info".into(),
            development_web_dir: None,
            telemetry_queue_capacity: 256,
            telemetry_batch_size: 64,
            telemetry_flush_milliseconds: 25,
            telemetry_enqueue_wait_milliseconds: 10,
            telemetry_request_timeout_milliseconds: 10_000,
            telemetry_shutdown_drain_milliseconds: 15_000,
            raw_retention_days: DEFAULT_RAW_RETENTION_DAYS,
            aggregate_retention_days: DEFAULT_AGGREGATE_RETENTION_DAYS,
            retention_interval_seconds: DEFAULT_MAINTENANCE_INTERVAL_SECONDS,
            retention_batch_size: DEFAULT_RETENTION_BATCH_SIZE,
            retention_max_transactions_per_run: DEFAULT_RETENTION_TRANSACTIONS,
            retention_max_run_milliseconds: DEFAULT_RETENTION_RUN_MILLISECONDS,
            retention_yield_milliseconds: DEFAULT_RETENTION_YIELD_MILLISECONDS,
        }
    }
}

pub const MAX_TELEMETRY_QUEUE_CAPACITY: usize = 1024;
pub const MAX_TELEMETRY_BATCH_SIZE: usize = 512;

pub const MAX_FLUSH_LATENCY: std::time::Duration = std::time::Duration::from_secs(1);
pub const MAX_ENQUEUE_WAIT: std::time::Duration = std::time::Duration::from_millis(250);
pub const MAX_REQUEST_TIME: std::time::Duration = std::time::Duration::from_secs(30);
pub const MAX_SHUTDOWN_DRAIN: std::time::Duration = std::time::Duration::from_secs(60);
pub const MIN_REQUEST_TIME: std::time::Duration = std::time::Duration::from_millis(100);
pub const MIN_SHUTDOWN_DRAIN: std::time::Duration = std::time::Duration::from_millis(100);

pub(crate) fn data_path_spec() -> serde_json::Value {
    serde_json::json!({"base":"config","value":"/data_dir","fallback":null,"suffix":"","encoding":"path"})
}

pub(crate) fn intrinsic_rules() -> serde_json::Value {
    use serde_json::json;
    let mut rules = vec![
        json!({"kind":"enum","fields":["/log_level"],"arguments":{"values":["off","error","warn","info","debug","trace"]}}),
        json!({"kind":"socket-address","fields":["/bind"],"arguments":{"loopback_when":"/development"}}),
        json!({"kind":"path","fields":["/data_dir"],"arguments":{"absolute":true}}),
        json!({"kind":"path","fields":["/development_web_dir"],"arguments":{"absolute":true,"optional":true}}),
    ];
    for (field, minimum, maximum) in [
        (
            "/telemetry_queue_capacity",
            1,
            MAX_TELEMETRY_QUEUE_CAPACITY as u64,
        ),
        ("/telemetry_batch_size", 1, MAX_TELEMETRY_BATCH_SIZE as u64),
        (
            "/telemetry_flush_milliseconds",
            1,
            MAX_FLUSH_LATENCY.as_millis() as u64,
        ),
        (
            "/telemetry_enqueue_wait_milliseconds",
            1,
            MAX_ENQUEUE_WAIT.as_millis() as u64,
        ),
        (
            "/telemetry_request_timeout_milliseconds",
            MIN_REQUEST_TIME.as_millis() as u64,
            MAX_REQUEST_TIME.as_millis() as u64,
        ),
        (
            "/telemetry_shutdown_drain_milliseconds",
            MIN_SHUTDOWN_DRAIN.as_millis() as u64,
            MAX_SHUTDOWN_DRAIN.as_millis() as u64,
        ),
        (
            "/raw_retention_days",
            MIN_RAW_RETENTION.as_secs() / 86400,
            MAX_RAW_RETENTION.as_secs() / 86400,
        ),
        (
            "/aggregate_retention_days",
            1,
            MAX_AGGREGATE_RETENTION.as_secs() / 86400,
        ),
        (
            "/retention_interval_seconds",
            MIN_MAINTENANCE_INTERVAL.as_secs(),
            MAX_MAINTENANCE_INTERVAL.as_secs(),
        ),
        ("/retention_batch_size", 1, MAX_RETENTION_BATCH_SIZE as u64),
        (
            "/retention_max_transactions_per_run",
            MIN_RETENTION_TRANSACTIONS as u64,
            MAX_RETENTION_TRANSACTIONS as u64,
        ),
        (
            "/retention_max_run_milliseconds",
            MIN_RETENTION_RUN_TIME.as_millis() as u64,
            MAX_RETENTION_RUN_TIME.as_millis() as u64,
        ),
        (
            "/retention_yield_milliseconds",
            1,
            MAX_RETENTION_YIELD.as_millis() as u64,
        ),
    ] {
        rules.push(json!({"kind":"integer-range","fields":[field],"arguments":{"minimum":minimum,"maximum":maximum}}));
    }
    rules.extend([
        json!({"kind":"integer-order","fields":["/telemetry_batch_size","/telemetry_queue_capacity"],"arguments":{"relation":"less-or-equal"}}),
        json!({"kind":"integer-order","fields":["/raw_retention_days","/aggregate_retention_days"],"arguments":{"relation":"less-than"}}),
        json!({"kind":"integer-order","fields":["/retention_max_run_milliseconds","/retention_interval_seconds"],"arguments":{"relation":"less-than","right_scale":1000}}),
        json!({"kind":"integer-sum-order","fields":["/telemetry_enqueue_wait_milliseconds","/telemetry_flush_milliseconds","/telemetry_request_timeout_milliseconds"],"arguments":{"relation":"less-than"}}),
    ]);
    json!(rules)
}

pub(crate) fn current_rules(source_bound: bool) -> serde_json::Value {
    use serde_json::json;
    let mut rules = intrinsic_rules().as_array().unwrap().clone();
    rules.extend([
        json!({"kind":"root-state-paths","fields":[],"arguments":{"paths":[data_path_spec()],"protected":xcss::server_layout::ROOT_RESERVED_DIRECTORIES}}),
        json!({"kind":"external-web-override","fields":["/development_web_dir"],"arguments":{"development_pointer":"/development","source_bound":source_bound}}),
    ]);
    json!(rules)
}

// This product's sum condition is part of its owned configuration grammar.
// The same finite declaration is exported for independent installation tools.
fn sum_order(rule: &serde_json::Value, value: &serde_json::Value) -> bool {
    let Some(fields) = rule["fields"].as_array() else {
        return false;
    };
    if !(3..=16).contains(&fields.len()) {
        return false;
    }
    let Some(arguments) = rule["arguments"].as_object() else {
        return false;
    };
    if arguments.len() != 1 {
        return false;
    }
    let mut values = Vec::with_capacity(fields.len());
    for field in fields {
        let Some(number) = field
            .as_str()
            .and_then(|field| value.pointer(field))
            .and_then(serde_json::Value::as_u64)
        else {
            return false;
        };
        values.push(number);
    }
    let right = values.pop().unwrap();
    let Some(left) = values.into_iter().try_fold(0u64, u64::checked_add) else {
        return false;
    };
    match arguments
        .get("relation")
        .and_then(serde_json::Value::as_str)
    {
        Some("less-than") => left < right,
        Some("less-or-equal") => left <= right,
        _ => false,
    }
}

pub(crate) fn validate_value(
    rules: &serde_json::Value,
    value: &serde_json::Value,
    root: &Path,
    source: xcss::config::ConfigSource,
) -> Result<(), xcss::config::ConfigError> {
    for rule in rules.as_array().ok_or_else(|| {
        xcss::config::ConfigError::new(xcss::config::Reason::InvalidMapping, "", source)
    })? {
        if rule["kind"] == "integer-sum-order" {
            if !sum_order(rule, value) {
                return Err(xcss::config::ConfigError::new(
                    xcss::config::Reason::InvalidValue,
                    "/telemetry_request_timeout_milliseconds",
                    source,
                ));
            }
        } else {
            xcss::configuration_contract::validate_rules(
                &serde_json::json!([rule]),
                value,
                root,
                source,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn validate_intrinsic(
    settings: &Settings,
    source: xcss::config::ConfigSource,
) -> Result<(), xcss::config::ConfigError> {
    let value = serde_json::to_value(settings).map_err(|_| {
        xcss::config::ConfigError::new(xcss::config::Reason::InvalidMapping, "", source)
    })?;
    validate_value(
        &intrinsic_rules(),
        &value,
        Path::new("/configuration-validation"),
        source,
    )
}

pub(crate) fn validate(
    settings: &Settings,
    root: &Path,
    source_bound: bool,
    source: xcss::config::ConfigSource,
) -> Result<(), xcss::config::ConfigError> {
    let value = serde_json::to_value(settings).map_err(|_| {
        xcss::config::ConfigError::new(xcss::config::Reason::InvalidMapping, "", source)
    })?;
    validate_value(&current_rules(source_bound), &value, root, source)
}

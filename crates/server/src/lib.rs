mod capacity;
pub mod config;
mod config_definition;
pub mod crypto;
mod database_definition;
pub mod database_lock;
pub mod database_schema;
pub mod error;
pub mod http;
pub mod model;
pub mod pagination;
mod pairing_admission;
mod persistent_crypto_definition;
pub mod release_contract;
pub mod retention;
mod retention_defaults;
pub mod runtime_validation;
mod secret_definition;
pub mod secrets;
pub mod store;
pub mod telemetry;

pub fn token_hash(token: &str) -> String {
    xcss::admin_auth::token_hash_hex(token)
}

mod hardware_validation;

pub mod web_assets;

#[cfg(test)]
#[path = "../tests/support/administrator.rs"]
mod test_support;

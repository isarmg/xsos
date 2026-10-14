//! Current secret document definition shared with installation material generation.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstanceSecrets {
    pub(crate) client_authorization_key: String,
}

impl InstanceSecrets {
    pub(crate) fn key(&self) -> anyhow::Result<[u8; 32]> {
        STANDARD
            .decode(&self.client_authorization_key)
            .ok()
            .and_then(|key| key.try_into().ok())
            .ok_or_else(|| anyhow::anyhow!("persisted client authorization key is invalid"))
    }
}

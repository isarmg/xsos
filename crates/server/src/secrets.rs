//! Persistent instance keys are initialized once and never recovered from configuration.

use std::path::Path;

#[cfg(test)]
use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::secret_definition::InstanceSecrets;

/// Create the private key store for an explicitly initialized empty instance.
#[cfg(test)]
pub fn initialize(data_dir: &Path) -> anyhow::Result<()> {
    let secrets = InstanceSecrets {
        client_authorization_key: STANDARD.encode(xcss::server_secrets::generate_key()?),
    };
    xcss::server_secrets::initialize(data_dir, &secrets)?;
    Ok(())
}

/// Read and validate the existing key store. This never generates or repairs keys.
pub fn load(data_dir: &Path) -> anyhow::Result<[u8; 32]> {
    let secrets: InstanceSecrets = xcss::server_secrets::load(data_dir)?;
    secrets.key()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_keys_require_exact_shape_and_length() {
        assert!(serde_json::from_str::<InstanceSecrets>(r#"{}"#).is_err());
        assert!(
            serde_json::from_value::<InstanceSecrets>(serde_json::json!({
                "client_authorization_key": STANDARD.encode([7_u8; 32]),
                "unknown": true
            }))
            .is_err()
        );
        for value in [
            "not-base64".into(),
            STANDARD.encode([7_u8; 31]),
            STANDARD.encode([7_u8; 33]),
        ] {
            assert!(
                InstanceSecrets {
                    client_authorization_key: value
                }
                .key()
                .is_err()
            );
        }
        assert_eq!(
            InstanceSecrets {
                client_authorization_key: STANDARD.encode([7_u8; 32])
            }
            .key()
            .unwrap(),
            [7_u8; 32]
        );
    }
}

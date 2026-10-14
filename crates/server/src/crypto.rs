use std::sync::Arc;

use xcss::secret::{SecretBytes, SecretKey};

use crate::persistent_crypto_definition::ClientAuthorizationEnvelope;

#[derive(Clone)]
pub struct SecretBox(Arc<SecretKey<32>>);

impl SecretBox {
    pub fn new(key: [u8; 32]) -> Self {
        Self(Arc::new(SecretKey::new(key)))
    }

    pub fn encrypt(&self, instance_id: uuid::Uuid, value: &str) -> anyhow::Result<Vec<u8>> {
        Ok(xcss::secret_envelope::seal::<ClientAuthorizationEnvelope>(
            &self.0,
            instance_id.as_bytes(),
            &SecretBytes::new(value.as_bytes().to_vec()),
        )?)
    }

    pub fn decrypt(&self, instance_id: uuid::Uuid, value: &[u8]) -> anyhow::Result<String> {
        anyhow::ensure!(
            (64..=1024).contains(&value.len()),
            "client authorization envelope has an invalid size"
        );
        let plaintext = xcss::secret_envelope::open::<ClientAuthorizationEnvelope>(
            &self.0,
            instance_id.as_bytes(),
            value,
        )?;
        Ok(String::from_utf8(plaintext.expose().to_vec())?)
    }
}

#[cfg(test)]
mod installation_vector_tests {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    #[test]
    fn current_authorization_material_matches_runtime_decryption() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../schema/validation/authorization.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            let input = &case["input"];
            let valid = (|| -> anyhow::Result<()> {
                let key: [u8; 32] = hex::decode(input["key_hex"].as_str().unwrap())?
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("bad key"))?;
                let identity = hex::decode(input["identity_hex"].as_str().unwrap())?;
                let identity = if identity.len() == 16 {
                    uuid::Uuid::from_slice(&identity)?
                } else {
                    uuid::Uuid::parse_str(std::str::from_utf8(&identity)?)?
                };
                let ciphertext = STANDARD.decode(input["ciphertext_base64"].as_str().unwrap())?;
                let plaintext = SecretBox::new(key).decrypt(identity, &ciphertext)?;
                crate::model::validate_stored_activation_code(&plaintext)?;
                let digest = hex::decode(input["digest_hex"].as_str().unwrap())?;
                anyhow::ensure!(
                    digest == crate::token_hash(&plaintext).as_bytes(),
                    "digest mismatch"
                );
                Ok(())
            })()
            .is_ok();
            assert_eq!(
                valid,
                case["valid"].as_bool().unwrap(),
                "runtime differs from installation case {}",
                case["name"]
            );
        }
    }
}

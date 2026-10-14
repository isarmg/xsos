//! Checks of already installed runtime state; never initializes or repairs.
use crate::store;
fn private_runtime_directories(root: &std::path::Path) -> anyhow::Result<()> {
    xcss::state_file::PrivateStateDirectory::open(root.join("logs"))?;
    xcss::state_file::PrivateStateDirectory::open(root.join("run"))?;
    Ok(())
}

/// Validate only existing runtime inputs before acquiring process locks.
/// Database contents are checked once, after both locks are held.
pub fn validate_runtime_inputs(config: &crate::config::ValidatedConfig) -> anyhow::Result<()> {
    private_runtime_directories(&config.root)?;
    xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
    crate::secrets::load(&config.data_dir)?;
    crate::database_schema::validate_existing_file(&crate::database_schema::database_path(
        &config.database_url,
    )?)?;
    Ok(())
}

pub async fn validate_state(config: &crate::config::ValidatedConfig) -> anyhow::Result<[u8; 32]> {
    crate::web_assets::verify()?;
    private_runtime_directories(&config.root)?;
    xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
    let key = crate::secrets::load(&config.data_dir)?;
    let snapshot = crate::database_schema::validation_snapshot(
        crate::database_schema::database_path(&config.database_url)?,
        config.root.join("run"),
    )
    .await?;
    let result = async {
        crate::database_schema::validate_pool(snapshot.pool()).await?;
        xcss::sqlite::integrity_check(snapshot.pool()).await?;
        xcss::sqlite::foreign_key_check(snapshot.pool()).await?;
        store::require_administrator(snapshot.pool()).await?;
        store::validate_invite_authorizations(snapshot.pool(), &crate::crypto::SecretBox::new(key))
            .await?;
        Ok::<_, anyhow::Error>(())
    }
    .await;
    snapshot.close().await;
    result?;
    Ok(key)
}

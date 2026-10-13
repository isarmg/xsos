//! Offline product state operations linked only by the separately signed setup helper.
use crate::{database_lock::MaintenanceLock, store};
use std::path::Path;
use xcss::contracts::SchemaIdentity;
use xcss::offline::{CheckReport, ConfigReport};
fn load(root: &Path, config: &Path) -> anyhow::Result<crate::config::ValidatedConfig> {
    crate::config::ValidatedConfig::from_root_sources(root, config)
}
fn private_runtime_directories(root: &Path) -> anyhow::Result<()> {
    xcss::state_file::PrivateStateDirectory::open(root.join("logs"))?;
    xcss::state_file::PrivateStateDirectory::open(root.join("run"))?;
    Ok(())
}
fn report(config: &crate::config::ValidatedConfig, path: &Path) -> anyhow::Result<ConfigReport> {
    let mut state_paths = vec![
        path.to_path_buf(),
        config.data_dir.clone(),
        config.root.join("logs"),
    ];
    state_paths.sort();
    state_paths.dedup();
    let mut writable_paths = vec![
        config.data_dir.clone(),
        config.root.join("logs"),
        config.root.join("run"),
    ];
    writable_paths.sort();
    writable_paths.dedup();
    Ok(ConfigReport {
        product: "xsos".into(),
        root: config.root.clone(),
        data_dir: config.data_dir.clone(),
        state_paths,
        schema_identity: crate::database_schema::expected_identity()?,
        bind: config.bind,
        writable_paths,
        shared_paths: Vec::new(),
    })
}
pub async fn validate_config(root: &Path, config: &Path) -> anyhow::Result<ConfigReport> {
    report(&load(root, config)?, config)
}
pub async fn check(root: &Path, config: &Path) -> anyhow::Result<CheckReport> {
    let value = load(root, config)?;
    validate_state(&value).await?;
    Ok(CheckReport {
        config: report(&value, config)?,
        initialized: true,
    })
}
pub async fn migrate(
    root: &Path,
    config: &Path,
    data_dir: &Path,
    source_schema: &SchemaIdentity,
    target_schema: &SchemaIdentity,
) -> anyhow::Result<()> {
    let expected = crate::database_schema::expected_identity()?;
    anyhow::ensure!(
        source_schema == target_schema && target_schema == &expected,
        "no explicit migration exists for this schema transition"
    );
    anyhow::ensure!(
        load(root, config)?.data_dir == data_dir,
        "target data_dir differs from the verified upgrade plan"
    );
    check(root, config).await?;
    Ok(())
}
pub async fn initialize(
    root: &Path,
    config: &Path,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let username = xcss::admin_auth::normalize_administrator_username(username)?;
    xcss::admin_auth::validate_password(password)?;
    let config = load(root, config)?;
    private_runtime_directories(root)?;
    xcss::server_cli::create_empty_private_directory(&config.data_dir)
        .map_err(xcss::server_cli::CliError)?;
    let directory = xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
    xcss::server_cli::runtime_allowed(&config.data_dir).map_err(xcss::server_cli::CliError)?;
    let _maintenance = directory.try_maintenance_lock()?;
    xcss::server_cli::runtime_allowed(&config.data_dir).map_err(xcss::server_cli::CliError)?;
    let database = crate::database_schema::database_path(&config.database_url)?;
    anyhow::ensure!(
        !database.try_exists()?,
        "initialization never overwrites an existing database"
    );
    let _database_lock = MaintenanceLock::exclusive(&config.database_url)?;
    crate::secrets::initialize(&config.data_dir)?;
    let pool = store::open_or_initialize(&config.database_url).await?;
    let result = async {
        store::ensure_admin_user(&pool, &username, Some(password)).await?;
        xcss::sqlite::checkpoint(&pool).await?;
        Ok::<_, anyhow::Error>(())
    }
    .await;
    pool.close().await;
    result
}
pub async fn reset_password(
    root: &Path,
    config: &Path,
    username: &str,
    password: &str,
) -> anyhow::Result<()> {
    let username = xcss::admin_auth::normalize_administrator_username(username)?;
    xcss::admin_auth::validate_password(password)?;
    let config = load(root, config)?;
    let directory = xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
    xcss::server_cli::runtime_allowed(&config.data_dir).map_err(xcss::server_cli::CliError)?;
    let _maintenance = directory.try_maintenance_lock()?;
    xcss::server_cli::runtime_allowed(&config.data_dir).map_err(xcss::server_cli::CliError)?;
    validate_state(&config).await?;
    let lock = MaintenanceLock::exclusive(&config.database_url)?;
    let pool = crate::database_schema::open_validated_location(&lock.database_url()).await?;
    let result = store::reset_admin_password(&pool, &username, password).await;
    pool.close().await;
    result
}

/// Validate only existing runtime inputs before acquiring process locks.
/// Database contents are checked once, after both locks are held.
#[doc(hidden)]
pub fn validate_runtime_inputs(config: &crate::config::ValidatedConfig) -> anyhow::Result<()> {
    private_runtime_directories(&config.root)?;
    xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
    crate::secrets::load(&config.data_dir)?;
    crate::database_schema::validate_existing_file(&crate::database_schema::database_path(
        &config.database_url,
    )?)?;
    Ok(())
}

#[doc(hidden)]
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

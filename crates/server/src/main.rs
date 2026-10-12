use anyhow::Context as _;
use clap::Parser;
use xsos::{
    config::{AdminCommand, Cli, Command, ConfigCommand},
    database_lock::{ApplicationLock, MaintenanceLock},
    http::{AppState, product_descriptor, router},
    release_bundle,
    retention::RetentionMaintenance,
    store,
    telemetry::TelemetryWriter,
};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let json = std::env::args_os().any(|argument| argument == "--json");
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            return if error.print().is_ok() {
                std::process::ExitCode::SUCCESS
            } else {
                std::process::ExitCode::FAILURE
            };
        }
        Err(_) => {
            let error = xcss::server_cli::ErrorEnvelope::with_code(
                xcss::server_cli::ErrorCode::new("invalid_cli_input").unwrap(),
                "Command arguments do not satisfy the current CLI contract; use --help.",
            );
            return xcss::server_cli::report_error(&error, json, 2);
        }
    };
    match execute(cli).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let envelope = if let Some(error) = error.downcast_ref::<xcss::config::ConfigError>() {
                error.envelope()
            } else if let Some(error) = error.downcast_ref::<xcss::server_cli::CliError>() {
                error.0.clone()
            } else if let Some(error) = error.downcast_ref::<xcss::state_file::Error>() {
                xcss::server_cli::state_error(error)
            } else if let Some(error) = error.downcast_ref::<xcss::server_cli::SnapshotError>() {
                xcss::server_cli::snapshot_error(error)
            } else {
                if !json {
                    eprintln!("{error:#}");
                }
                xcss::server_cli::ErrorEnvelope::with_code(
                    xcss::server_cli::ErrorCode::new("current_state_invalid").unwrap(),
                    "The command could not validate or operate on the current configuration and data.",
                )
            };
            xcss::server_cli::report_error(&envelope, json, 1)
        }
    }
}

async fn query_status(bind: std::net::SocketAddr, json: bool) -> anyhow::Result<()> {
    let report = xcss::server_cli::query_status(bind, "xsos")
        .await
        .map_err(xcss::server_cli::CliError)?;
    if !report.ready {
        return Err(
            xcss::server_cli::CliError(xcss::server_cli::ErrorEnvelope::with_code(
                xcss::server_cli::ErrorCode::new("service_not_ready").unwrap(),
                "The service answered but its business readiness checks failed.",
            ))
            .into(),
        );
    }
    xcss::server_cli::print_report(&report, json)?;
    Ok(())
}

async fn execute(mut cli: Cli) -> anyhow::Result<()> {
    if !matches!(
        &cli.command,
        Command::Identity | Command::WebAssets | Command::VerifyRelease(_)
    ) {
        cli.config = xsos::config::selected_config_path(cli.config.as_deref())?;
    }
    xsos::release_contract::ensure_supported_runtime()?;
    xcss::server_runtime::install_panic_hook();
    let configuration = || {
        xsos::config::ValidatedConfig::from_sources(
            cli.config.as_deref(),
            cli.data_dir.as_deref(),
            cli.bind,
            cli.development,
        )
    };
    let config = match &cli.command {
        Command::Identity | Command::WebAssets | Command::VerifyRelease(_) => None,
        _ => Some(configuration()?),
    };
    initialize_logging(
        config
            .as_ref()
            .map_or("info", |config| config.log_level.as_str()),
    )?;
    match cli.command {
        Command::Init { username } => {
            let config = config.expect("command requires configuration");
            let username = store::normalize_username(&username)?;
            let password = read_password_from_stdin()?;
            xcss::admin_auth::validate_password(&password)?;
            xcss::server_cli::create_empty_private_directory(&config.data_dir)
                .map_err(xcss::server_cli::CliError)?;
            let directory = xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
            xcss::server_cli::runtime_allowed(directory.path())
                .map_err(xcss::server_cli::CliError)?;
            let _maintenance = directory.try_maintenance_lock()?;
            xcss::server_cli::runtime_allowed(directory.path())
                .map_err(xcss::server_cli::CliError)?;
            xcss::server_cli::create_runtime_log_directory(&config.data_dir)
                .map_err(xcss::server_cli::CliError)?;
            let database = xsos::database_schema::database_path(&config.database_url)?;
            anyhow::ensure!(
                !database.try_exists()?,
                "database already exists; init never overwrites existing data"
            );
            let _database_lock = MaintenanceLock::exclusive(&config.database_url)?;
            xsos::secrets::initialize(&config.data_dir)?;
            let pool = store::open_or_initialize(&config.database_url).await?;
            store::ensure_admin_user(&pool, &username, Some(&password)).await?;
            xcss::sqlite::checkpoint(&pool).await?;
            pool.close().await;
            tracing::info!(event = "common.initialization.completed");
            println!(
                "{}",
                serde_json::json!({"status":"initialized", "ready":false})
            );
        }
        Command::Run => {
            serve_with_config(config.expect("run requires configuration")).await?;
        }
        Command::Config {
            command: ConfigCommand::Validate,
        } => {
            let config = config.expect("command requires configuration");
            println!(
                "{}",
                serde_json::json!({"status":"valid", "state_paths":std::iter::once(config.data_dir.clone()).chain(cli.config.as_ref().map(|path| path.canonicalize()).transpose()?).collect::<Vec<_>>(), "sources":config.sources, "schema_identity":xsos::database_schema::expected_identity()?})
            );
        }
        Command::Status => {
            query_status(
                config.expect("status requires configuration").bind,
                cli.json,
            )
            .await?;
        }
        Command::Admin {
            command: AdminCommand::ResetPassword { username },
        } => {
            let config = config.expect("admin requires configuration");
            let directory = xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
            xcss::server_cli::runtime_allowed(directory.path())
                .map_err(xcss::server_cli::CliError)?;
            let _common_lock = directory.try_maintenance_lock()?;
            xcss::server_cli::runtime_allowed(directory.path())
                .map_err(xcss::server_cli::CliError)?;
            xsos::database_schema::validate_configuration_database(&config.database_url)?;
            let maintenance = MaintenanceLock::exclusive(&config.database_url)?;
            validate_existing_configuration(&config).await?;
            let pool =
                xsos::database_schema::open_validated_location(&maintenance.database_url()).await?;
            let username = store::normalize_username(&username)?;
            let password = read_password_from_stdin()?;
            store::reset_admin_password(&pool, &username, &password).await?;
            println!(
                "{{\"status\":\"password-reset\",\"username\":{:?}}}",
                username
            );
        }
        Command::Check => {
            let config = config.expect("check requires configuration");
            xcss::server_cli::runtime_allowed(&config.data_dir)
                .map_err(xcss::server_cli::CliError)?;
            validate_existing_configuration(&config).await?;
            let directory = xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
            xcss::server_cli::runtime_allowed(directory.path())
                .map_err(xcss::server_cli::CliError)?;
            let _common_lock = directory.try_maintenance_lock()?;
            xcss::server_cli::runtime_allowed(directory.path())
                .map_err(xcss::server_cli::CliError)?;
            let _maintenance = MaintenanceLock::exclusive(&config.database_url)?;
            validate_existing_configuration(&config).await?;
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "initialized": true,
                    "ready": false,
                    "schema_identity": xsos::database_schema::expected_identity()?,
                })
            );
        }
        Command::WebAssets => {
            xsos::web_assets::verify()?;
            println!("{}", xsos::web_assets::MANIFEST);
        }
        Command::Identity => {
            println!("{}", xsos::release_contract::current_json()?);
        }
        Command::VerifyRelease(args) => {
            let report = release_bundle::verify_release(&args.root)?;
            println!("{}", serde_json::to_string(&report)?);
        }
    }
    Ok(())
}

async fn serve_with_config(config: xsos::config::ValidatedConfig) -> anyhow::Result<()> {
    if config.administrator_origin
        == xcss::admin_auth::AdministratorOriginMode::LoopbackDevelopmentHttp
    {
        tracing::warn!(
            "XSOS_DEVELOPMENT is enabled; using an insecure loopback-only session cookie"
        );
    }
    let signals = xcss::server_runtime::ProcessSignals::install()?;
    let listeners = xcss::server_runtime::BoundListeners::bind([config.bind])?;
    let transport = xcss::server_runtime::HttpServer::new(listeners, signals);
    xcss::server_cli::runtime_allowed(&config.data_dir).map_err(xcss::server_cli::CliError)?;
    validate_existing_configuration(&config).await?;
    let directory = xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
    xcss::server_cli::runtime_allowed(directory.path()).map_err(xcss::server_cli::CliError)?;
    let _common_lock = directory.try_instance_lock()?;
    xcss::server_cli::runtime_allowed(directory.path()).map_err(xcss::server_cli::CliError)?;
    let application_lock = ApplicationLock::acquire(&config.database_url)?;
    let key = validate_existing_configuration(&config).await?;
    enable_runtime_logging(&config.data_dir)?;
    tracing::info!(event = "common.config.loaded");
    xsos::database_schema::validate_configuration_database(&config.database_url)?;
    let pool =
        xsos::database_schema::open_validated_location(&application_lock.database_url()).await?;
    store::require_administrator(&pool).await?;
    let secrets = xsos::crypto::SecretBox::new(key);
    store::validate_invite_authorizations(&pool, &secrets).await?;
    let (retention_status, retention_maintenance) =
        RetentionMaintenance::start(pool.clone(), config.retention);
    let (telemetry, telemetry_writer) = TelemetryWriter::start(pool.clone(), config.telemetry);
    let health_pool = pool.clone();
    let retention_pool = pool.clone();
    let runtime = xcss::server_runtime::ServerRuntime::builder(product_descriptor())
        .with_schema_identity(xsos::database_schema::expected_identity()?)
        .register_health_check(
            "database",
            xcss::server_runtime::health_check(move || {
                let pool = health_pool.clone();
                async move { store::ready(&pool).await }
            }),
        )
        .register_health_check(
            "retention-schema",
            xcss::server_runtime::health_check(move || {
                let pool = retention_pool.clone();
                async move { store::retention_ready(&pool).await }
            }),
        )
        .register_health_check(
            "retention-worker",
            xcss::server_runtime::health_check(move || {
                let status = retention_status.clone();
                async move { status.is_healthy() }
            }),
        )
        .register_background_task(
            "telemetry-writer",
            xcss::server_runtime::TaskCriticality::Critical,
            move |shutdown| telemetry_writer.run_until(shutdown),
        )
        .register_background_task(
            "telemetry-retention",
            xcss::server_runtime::TaskCriticality::Degrading,
            move |shutdown| retention_maintenance.run_until(shutdown),
        )
        .register_background_task(
            "shutdown-log",
            xcss::server_runtime::TaskCriticality::Degrading,
            |mut shutdown| async move {
                if !*shutdown.borrow() {
                    let _ = shutdown.changed().await;
                }
                tracing::info!(event = "common.runtime.shutdown_started");
                Ok(())
            },
        )
        .build()
        .await?;
    let runtime_handle = runtime.handle();
    tracing::info!(event = "common.runtime.started");
    let state = AppState::with_runtime(
        pool,
        config.administrator_origin,
        telemetry,
        runtime_handle.clone(),
        secrets,
    )
    .with_trusted_proxies(config.trusted_proxies);
    tracing::info!(bind=%config.bind, "xsos server ready");
    runtime
        .serve(
            transport,
            router(state, config.static_dir)?.layer(axum::middleware::from_fn_with_state(
                "xsos".to_owned(),
                xcss::server_cli::service_identity_middleware,
            )),
        )
        .await?;
    tracing::info!(event = "common.runtime.stopped");
    Ok(())
}

fn read_password_from_stdin() -> anyhow::Result<String> {
    use std::io::Read as _;

    let mut bytes = Vec::new();
    std::io::stdin()
        .take(4097)
        .read_to_end(&mut bytes)
        .context("read administrator password from standard input")?;
    anyhow::ensure!(
        bytes.len() <= 4096,
        "administrator password input is too large"
    );
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }
    anyhow::ensure!(
        !bytes.contains(&b'\n') && !bytes.contains(&b'\r'),
        "administrator password must be one line"
    );
    String::from_utf8(bytes).context("administrator password must be UTF-8")
}

static LOG_LAYER: std::sync::OnceLock<xcss::log::XcssStructuredLayer> = std::sync::OnceLock::new();
fn initialize_logging(log_level: &str) -> anyhow::Result<()> {
    use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};
    let layer = xcss::log::XcssStructuredLayer::new("xsos")?;
    LOG_LAYER
        .set(layer.clone())
        .map_err(|_| anyhow::anyhow!("logging already initialized"))?;
    tracing_subscriber::registry()
        .with(log_level.parse::<tracing_subscriber::filter::LevelFilter>()?)
        .with(layer)
        .try_init()?;
    Ok(())
}
fn enable_runtime_logging(data_dir: &std::path::Path) -> anyhow::Result<()> {
    xcss::server_cli::validate_runtime_log_directory(data_dir)
        .map_err(xcss::server_cli::CliError)?;
    let file = xcss::log::RotatingLogFile::open(
        data_dir.join("logs"),
        "xsos",
        xcss::log::LogRetention::default(),
    )?;
    LOG_LAYER
        .get()
        .ok_or_else(|| anyhow::anyhow!("logging is unavailable"))?
        .set_rotating_file(file)?;
    Ok(())
}

async fn validate_existing_configuration(
    config: &xsos::config::ValidatedConfig,
) -> anyhow::Result<[u8; 32]> {
    xcss::server_cli::validate_runtime_log_directory(&config.data_dir)
        .map_err(xcss::server_cli::CliError)?;
    xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
    let key = xsos::secrets::load(&config.data_dir)?;
    let snapshot = xsos::database_schema::validation_snapshot(
        xsos::database_schema::database_path(&config.database_url)?,
    )
    .await?;
    xsos::database_schema::validate_pool(snapshot.pool()).await?;
    xcss::sqlite::integrity_check(snapshot.pool()).await?;
    xcss::sqlite::foreign_key_check(snapshot.pool()).await?;
    store::require_administrator(snapshot.pool()).await?;
    store::validate_invite_authorizations(snapshot.pool(), &xsos::crypto::SecretBox::new(key))
        .await?;
    snapshot.close().await;
    Ok(key)
}

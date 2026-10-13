use clap::Parser;
use xsos::{
    config::Cli,
    database_lock::ApplicationLock,
    http::{AppState, product_descriptor, router},
    retention::RetentionMaintenance,
    store,
    telemetry::TelemetryWriter,
};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    let result = async {
        let config_path = cli.root.join("config/config.json");
        let config = xsos::config::ValidatedConfig::from_root_sources(&cli.root, &config_path)?;
        xsos::release_contract::ensure_supported_runtime()?;
        xcss::server_runtime::install_panic_hook();
        initialize_logging(&config.log_level)?;
        serve_with_config(config).await
    }
    .await;
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xsos could not start: {error}");
            std::process::ExitCode::FAILURE
        }
    }
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
    xsos::offline::validate_runtime_inputs(&config)?;
    let directory = xcss::state_file::PrivateStateDirectory::open(&config.data_dir)?;
    xcss::server_cli::runtime_allowed(directory.path()).map_err(xcss::server_cli::CliError)?;
    let _common_lock = directory.try_instance_lock()?;
    xcss::server_cli::runtime_allowed(directory.path()).map_err(xcss::server_cli::CliError)?;
    let application_lock = ApplicationLock::acquire(&config.database_url)?;
    let key = xsos::offline::validate_state(&config).await?;
    enable_runtime_logging(&config.root)?;
    tracing::info!(event = "common.config.loaded");
    let pool =
        xsos::database_schema::open_validated_location(&application_lock.database_url()).await?;
    let secrets = xsos::crypto::SecretBox::new(key);
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
    );
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
fn enable_runtime_logging(root: &std::path::Path) -> anyhow::Result<()> {
    xcss::state_file::PrivateStateDirectory::open(root.join("logs"))?;
    let file = xcss::log::RotatingLogFile::open(
        root.join("logs"),
        "xsos",
        xcss::log::LogRetention::default(),
    )?;
    LOG_LAYER
        .get()
        .ok_or_else(|| anyhow::anyhow!("logging is unavailable"))?
        .set_rotating_file(file)?;
    Ok(())
}

use std::{
    collections::BTreeMap,
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use xcss::config::{ConfigSource, EnvMapping, EnvValueKind, Override};

use crate::retention::{
    DEFAULT_AGGREGATE_RETENTION_DAYS, DEFAULT_MAINTENANCE_INTERVAL_SECONDS,
    DEFAULT_RAW_RETENTION_DAYS, DEFAULT_RETENTION_BATCH_SIZE, DEFAULT_RETENTION_RUN_MILLISECONDS,
    DEFAULT_RETENTION_TRANSACTIONS, DEFAULT_RETENTION_YIELD_MILLISECONDS, RetentionConfig,
};
use crate::telemetry::TelemetryWriterConfig;
use xcss::admin_auth::AdministratorOriginMode;

#[derive(Debug, Parser)]
#[command(name = "xsos", version,
    long_version = concat!(env!("CARGO_PKG_VERSION"), " source=", env!("XSOS_SOURCE_REVISION"), " xcss=", env!("XCSS_REVISION")), about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    #[arg(long, global = true)]
    pub data_dir: Option<PathBuf>,
    #[arg(long, global = true)]
    pub bind: Option<SocketAddr>,
    #[arg(long, global = true, action = clap::ArgAction::Set)]
    pub development: Option<bool>,
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Explicitly initialize a private data directory and first administrator.
    Init {
        #[arg(long, default_value = "admin")]
        username: String,
    },
    /// Run a previously initialized instance.
    Run,
    /// Validate configuration without requiring initialized instance data.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Query the live service readiness; offline instances never report ready.
    Status,
    /// Validate initialized instance data, keys, and directories while offline.
    Check,
    /// Print the exact machine-readable binary release identity.
    Identity,
    /// Print the immutable browser asset inventory compiled into this binary.
    WebAssets,
    /// Verify an immutable release tree with the binary contained in that tree.
    VerifyRelease(ReleaseRoot),
    /// Manage the initialized local administrator.
    Admin {
        #[command(subcommand)]
        command: AdminCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Validate,
}

#[derive(Debug, Clone, clap::Args)]
pub struct ReleaseRoot {
    #[arg(long)]
    pub root: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum AdminCommand {
    /// Reset an existing administrator password, read from standard input.
    ResetPassword {
        #[arg(long)]
        username: String,
    },
}

#[derive(Clone)]
pub struct ValidatedConfig {
    pub bind: SocketAddr,
    pub database_url: String,
    pub administrator_origin: AdministratorOriginMode,
    pub telemetry: TelemetryWriterConfig,
    pub retention: RetentionConfig,
    pub static_dir: Option<PathBuf>,
    pub log_level: String,
    pub trusted_proxies: xcss::client_ip::TrustedProxies,
    pub data_dir: PathBuf,
    pub sources: BTreeMap<String, ConfigSource>,
}

impl ValidatedConfig {
    pub fn from_runtime() -> anyhow::Result<Self> {
        Self::from_sources(None, None, None, None)
    }

    pub fn from_sources(
        config: Option<&Path>,
        data_dir: Option<&Path>,
        bind: Option<SocketAddr>,
        development: Option<bool>,
    ) -> anyhow::Result<Self> {
        let config = selected_config_path(config)?;
        let file = config
            .as_deref()
            .map(xcss::config::read_private_file)
            .transpose()?;
        let environment = xcss::config::read_environment(&ENVIRONMENT, |name| env::var(name).ok())?;
        let mut command_line = Vec::new();
        if let Some(path) = data_dir {
            command_line.push(Override::new(
                "/data_dir",
                path.to_string_lossy().into_owned(),
            ));
        }
        if let Some(bind) = bind {
            command_line.push(Override::new("/bind", bind.to_string()));
        }
        if let Some(development) = development {
            command_line.push(Override::new("/development", development));
        }
        let loaded = xcss::config::resolve_validated(
            &Settings::default(),
            file.as_deref(),
            &environment,
            &command_line,
            validate_intrinsic,
        )?;
        let settings = loaded.value;
        let data_dir = settings.data_dir;
        let database_url = format!("sqlite://{}", data_dir.join("xsos.sqlite3").display());
        crate::database_schema::database_path(&database_url)?;
        let bind = settings.bind;
        let cookie_mode = cookie_mode(bind, settings.development)?;
        let static_dir = development_static_dir(
            settings.development_web_dir.as_deref(),
            settings.development,
        )?;
        let telemetry = TelemetryWriterConfig::new(
            settings.telemetry_queue_capacity,
            settings.telemetry_batch_size,
            Duration::from_millis(settings.telemetry_flush_milliseconds),
            Duration::from_millis(settings.telemetry_enqueue_wait_milliseconds),
            Duration::from_millis(settings.telemetry_request_timeout_milliseconds),
            Duration::from_millis(settings.telemetry_shutdown_drain_milliseconds),
        )?;
        let days = |value: u64| {
            value
                .checked_mul(86400)
                .map(Duration::from_secs)
                .ok_or_else(|| anyhow::anyhow!("retention days exceed the supported duration"))
        };
        let retention = RetentionConfig::new(
            days(settings.raw_retention_days)?,
            days(settings.aggregate_retention_days)?,
            Duration::from_secs(settings.retention_interval_seconds),
            settings.retention_batch_size,
            settings.retention_max_transactions_per_run,
            Duration::from_millis(settings.retention_max_run_milliseconds),
            Duration::from_millis(settings.retention_yield_milliseconds),
        )?;
        Ok(Self {
            bind,
            database_url,
            administrator_origin: cookie_mode,
            telemetry,
            retention,
            static_dir,
            log_level: settings.log_level,
            trusted_proxies: xcss::client_ip::TrustedProxies::parse(&settings.trusted_proxies)?,
            data_dir,
            sources: loaded.sources,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    data_dir: PathBuf,
    bind: SocketAddr,
    development: bool,
    log_level: String,
    trusted_proxies: Vec<String>,
    development_web_dir: Option<String>,
    telemetry_queue_capacity: usize,
    telemetry_batch_size: usize,
    telemetry_flush_milliseconds: u64,
    telemetry_enqueue_wait_milliseconds: u64,
    telemetry_request_timeout_milliseconds: u64,
    telemetry_shutdown_drain_milliseconds: u64,
    raw_retention_days: u64,
    aggregate_retention_days: u64,
    retention_interval_seconds: u64,
    retention_batch_size: usize,
    retention_max_transactions_per_run: usize,
    retention_max_run_milliseconds: u64,
    retention_yield_milliseconds: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("/var/lib/isarmg/xsos"),
            bind: "127.0.0.1:18105".parse().expect("valid default bind"),
            development: false,
            log_level: "info".into(),
            trusted_proxies: Vec::new(),
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

const ENVIRONMENT: [EnvMapping<'static>; 19] = [
    EnvMapping {
        variable: "XSOS_DATA_DIR",
        path: "/data_dir",
        kind: EnvValueKind::String,
    },
    EnvMapping {
        variable: "XSOS_BIND",
        path: "/bind",
        kind: EnvValueKind::String,
    },
    EnvMapping {
        variable: "XSOS_DEVELOPMENT",
        path: "/development",
        kind: EnvValueKind::Boolean,
    },
    EnvMapping {
        variable: "XSOS_TRUSTED_PROXIES",
        path: "/trusted_proxies",
        kind: EnvValueKind::Json,
    },
    EnvMapping {
        variable: "XSOS_LOG_LEVEL",
        path: "/log_level",
        kind: EnvValueKind::String,
    },
    EnvMapping {
        variable: "XSOS_DEVELOPMENT_WEB_DIR",
        path: "/development_web_dir",
        kind: EnvValueKind::String,
    },
    EnvMapping {
        variable: "XSOS_TELEMETRY_QUEUE_CAPACITY",
        path: "/telemetry_queue_capacity",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_TELEMETRY_BATCH_SIZE",
        path: "/telemetry_batch_size",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_TELEMETRY_FLUSH_MILLISECONDS",
        path: "/telemetry_flush_milliseconds",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_TELEMETRY_ENQUEUE_WAIT_MILLISECONDS",
        path: "/telemetry_enqueue_wait_milliseconds",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_TELEMETRY_REQUEST_TIMEOUT_MILLISECONDS",
        path: "/telemetry_request_timeout_milliseconds",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_TELEMETRY_SHUTDOWN_DRAIN_MILLISECONDS",
        path: "/telemetry_shutdown_drain_milliseconds",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_RAW_RETENTION_DAYS",
        path: "/raw_retention_days",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_AGGREGATE_RETENTION_DAYS",
        path: "/aggregate_retention_days",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_RETENTION_INTERVAL_SECONDS",
        path: "/retention_interval_seconds",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_RETENTION_BATCH_SIZE",
        path: "/retention_batch_size",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_RETENTION_MAX_TRANSACTIONS_PER_RUN",
        path: "/retention_max_transactions_per_run",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_RETENTION_MAX_RUN_MILLISECONDS",
        path: "/retention_max_run_milliseconds",
        kind: EnvValueKind::UnsignedInteger,
    },
    EnvMapping {
        variable: "XSOS_RETENTION_YIELD_MILLISECONDS",
        path: "/retention_yield_milliseconds",
        kind: EnvValueKind::UnsignedInteger,
    },
];

fn development_static_dir(
    value: Option<&str>,
    development: bool,
) -> anyhow::Result<Option<PathBuf>> {
    let Some(value) = value else { return Ok(None) };
    anyhow::ensure!(
        development && crate::release_contract::SOURCE_REVISION == "unbound",
        "directory Web assets are allowed only in an unbound development build"
    );
    Ok(Some(validate_static_dir(value, false)?))
}

fn validate_static_dir(value: &str, production: bool) -> anyhow::Result<PathBuf> {
    anyhow::ensure!(
        !production,
        "production Web assets are embedded in the binary"
    );
    let root = Path::new(value);
    anyhow::ensure!(
        root.is_absolute(),
        "development Web directory must be absolute"
    );
    xcss::web_assets::DirectoryAssets::new(root)?;
    anyhow::ensure!(
        root.join("index.html").is_file() && root.join("assets").is_dir(),
        "development Web directory must contain index.html and assets"
    );
    let mut entries = fs::read_dir(root)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    anyhow::ensure!(
        entries
            == [
                std::ffi::OsString::from("assets"),
                std::ffi::OsString::from("index.html")
            ],
        "development Web directory must contain only the declared browser entry and assets"
    );
    Ok(fs::canonicalize(root)?)
}

fn cookie_mode(bind: SocketAddr, development: bool) -> anyhow::Result<AdministratorOriginMode> {
    if development && !bind.ip().is_loopback() {
        anyhow::bail!("XSOS_DEVELOPMENT requires a loopback XSOS_BIND");
    }
    Ok(if development {
        AdministratorOriginMode::LoopbackDevelopmentHttp
    } else {
        AdministratorOriginMode::ProductionHttps
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_assets_need_no_external_directory_and_production_rejects_overrides() {
        assert!(development_static_dir(None, false).unwrap().is_none());
        assert!(development_static_dir(Some("/not-needed"), false).is_err());
    }

    #[test]
    fn insecure_development_cookie_mode_is_explicit_and_loopback_only() {
        assert_eq!(
            cookie_mode("127.0.0.1:18105".parse().unwrap(), false).unwrap(),
            AdministratorOriginMode::ProductionHttps
        );
        assert_eq!(
            cookie_mode("127.0.0.1:18105".parse().unwrap(), true).unwrap(),
            AdministratorOriginMode::LoopbackDevelopmentHttp
        );
        assert!(cookie_mode("0.0.0.0:18105".parse().unwrap(), true).is_err());
    }

    #[test]
    fn product_cli_rejects_unknown_maintenance_commands() {
        for removed in [
            "migrate",
            "backup-create",
            "backup-verify",
            "restore",
            "doctor",
            "admin-reset-password",
        ] {
            assert!(
                Cli::try_parse_from(["xsos", removed]).is_err(),
                "removed product command {removed} was still accepted"
            );
        }
    }

    #[test]
    fn administrator_username_uses_the_foundation_canonical_form() {
        assert_eq!(
            xcss::admin_auth::normalize_administrator_username("  Release.Admin  ").unwrap(),
            "release.admin"
        );
        for rejected in ["ab", "admin@example.test", "管理员", "admin\n"] {
            assert!(xcss::admin_auth::normalize_administrator_username(rejected).is_err());
        }
    }

    #[test]
    fn immutable_release_commands_require_an_explicit_root() {
        let root = "/opt/isarmg/xsos/releases/0.12.0";
        assert!(Cli::try_parse_from(["xsos", "run"]).is_ok());
        assert!(Cli::try_parse_from(["xsos", "run", "--release-root", root]).is_err());
        assert!(Cli::try_parse_from(["xsos", "verify-release", "--root", root]).is_ok());
        assert!(Cli::try_parse_from(["xsos", "run", "--release-root"]).is_err());
        assert!(Cli::try_parse_from(["xsos", "verify-release"]).is_err());
    }

    #[test]
    fn static_directory_is_absolute_and_exact() {
        let directory = tempfile::tempdir().unwrap();
        fs::write(directory.path().join("index.html"), "current").unwrap();
        fs::create_dir(directory.path().join("assets")).unwrap();
        fs::write(directory.path().join("assets/app.js"), "current").unwrap();

        assert_eq!(
            validate_static_dir(directory.path().to_str().unwrap(), false).unwrap(),
            directory.path().canonicalize().unwrap()
        );
        assert!(validate_static_dir(directory.path().to_str().unwrap(), true).is_err());
        assert!(validate_static_dir("web/dist", false).is_err());
        fs::write(directory.path().join("unexpected"), "not current").unwrap();
        assert!(validate_static_dir(directory.path().to_str().unwrap(), false).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn static_directory_rejects_symbolic_and_hard_linked_assets() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let index = directory.path().join("index.html");
        fs::write(&index, "current").unwrap();
        fs::create_dir(directory.path().join("assets")).unwrap();
        fs::write(directory.path().join("assets/app.js"), "current").unwrap();
        let alias = directory.path().join("alias.html");
        fs::hard_link(&index, &alias).unwrap();
        assert!(validate_static_dir(directory.path().to_str().unwrap(), false).is_err());

        fs::remove_file(&alias).unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        symlink(outside.path(), directory.path().join("linked.js")).unwrap();
        assert!(validate_static_dir(directory.path().to_str().unwrap(), false).is_err());
    }
}

fn validate_intrinsic(
    settings: &Settings,
    source: ConfigSource,
) -> Result<(), xcss::config::ConfigError> {
    let invalid =
        |path| xcss::config::ConfigError::new(xcss::config::Reason::InvalidValue, path, source);

    if !settings.data_dir.is_absolute()
        || settings
            .data_dir
            .components()
            .any(|component| component == std::path::Component::ParentDir)
    {
        return Err(invalid("/data_dir"));
    }
    if !matches!(
        settings.log_level.as_str(),
        "trace" | "debug" | "info" | "warn" | "error" | "off"
    ) {
        return Err(invalid("/log_level"));
    }
    xcss::client_ip::TrustedProxies::parse(&settings.trusted_proxies)
        .map_err(|_| invalid("/trusted_proxies"))?;
    for (path, value, min, max) in [
        (
            "/telemetry_queue_capacity",
            settings.telemetry_queue_capacity as u64,
            1,
            crate::telemetry::MAX_QUEUE_CAPACITY as u64,
        ),
        (
            "/telemetry_batch_size",
            settings.telemetry_batch_size as u64,
            1,
            crate::telemetry::MAX_BATCH_SIZE as u64,
        ),
        (
            "/retention_batch_size",
            settings.retention_batch_size as u64,
            1,
            crate::retention::MAX_RETENTION_BATCH_SIZE as u64,
        ),
        (
            "/retention_max_transactions_per_run",
            settings.retention_max_transactions_per_run as u64,
            3,
            crate::retention::MAX_RETENTION_TRANSACTIONS as u64,
        ),
        (
            "/telemetry_flush_milliseconds",
            settings.telemetry_flush_milliseconds,
            1,
            1000,
        ),
        (
            "/telemetry_enqueue_wait_milliseconds",
            settings.telemetry_enqueue_wait_milliseconds,
            1,
            250,
        ),
        (
            "/telemetry_request_timeout_milliseconds",
            settings.telemetry_request_timeout_milliseconds,
            100,
            30000,
        ),
        (
            "/telemetry_shutdown_drain_milliseconds",
            settings.telemetry_shutdown_drain_milliseconds,
            100,
            60000,
        ),
        ("/raw_retention_days", settings.raw_retention_days, 1, 365),
        (
            "/aggregate_retention_days",
            settings.aggregate_retention_days,
            1,
            3650,
        ),
        (
            "/retention_interval_seconds",
            settings.retention_interval_seconds,
            1,
            86400,
        ),
        (
            "/retention_max_run_milliseconds",
            settings.retention_max_run_milliseconds,
            100,
            10000,
        ),
        (
            "/retention_yield_milliseconds",
            settings.retention_yield_milliseconds,
            1,
            100,
        ),
    ] {
        if !(min..=max).contains(&value) {
            return Err(invalid(path));
        }
    }
    Ok(())
}

#[cfg(test)]
mod precedence_contract_tests {
    use super::*;
    #[test]
    fn a_higher_priority_override_cannot_hide_an_invalid_file_value() {
        let file = serde_json::to_vec(&serde_json::json!({"telemetry_queue_capacity":0})).unwrap();
        let error = xcss::config::resolve_validated(
            &Settings::default(),
            Some(&file),
            &[],
            &[Override::new(
                "/telemetry_queue_capacity",
                serde_json::json!(16),
            )],
            validate_intrinsic,
        )
        .err()
        .expect("invalid lower layer must fail");
        assert_eq!(error.path, "/telemetry_queue_capacity");
        assert_eq!(error.source, ConfigSource::File);
        assert!(
            !serde_json::to_string(&error.envelope())
                .unwrap()
                .contains("private-invalid-secret")
        );
    }
}

/// Normalize the CLI file authority before private reads and resource reporting.
pub fn normalize_config_path(path: &Path) -> anyhow::Result<PathBuf> {
    anyhow::ensure!(
        !path
            .components()
            .any(|part| part == std::path::Component::ParentDir),
        "config path cannot contain parent traversal"
    );
    Ok(std::path::absolute(path)?)
}

#[cfg(test)]
mod unified_configuration_tests {
    use super::*;
    #[test]
    fn explicit_missing_config_is_an_error() {
        let directory = tempfile::tempdir().unwrap();
        let missing = directory.path().join("missing.json");
        assert!(ValidatedConfig::from_sources(Some(&missing), None, None, None).is_err());
    }
    #[test]
    fn defaults_and_environment_belong_to_this_server() {
        let settings = Settings::default();
        assert!(!settings.development);
        assert_eq!(settings.data_dir, PathBuf::from("/var/lib/isarmg/xsos"));
        assert!(
            ENVIRONMENT
                .iter()
                .all(|mapping| mapping.variable.starts_with("XSOS_"))
        );
        assert!(
            !ENVIRONMENT
                .iter()
                .any(|mapping| mapping.path.contains("bootstrap"))
        );
    }
    #[test]
    fn runtime_settings_reject_removed_secret_and_database_authorities() {
        for file in [
            br#"{"database_url":"sqlite:///tmp/other.sqlite3"}"#.as_slice(),
            br#"{"client_authorization_key":"BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc="}"#
                .as_slice(),
        ] {
            assert!(
                xcss::config::resolve_validated(
                    &Settings::default(),
                    Some(file),
                    &[],
                    &[],
                    validate_intrinsic
                )
                .is_err()
            );
        }
        assert!(!ENVIRONMENT.iter().any(|mapping| matches!(
            mapping.variable,
            "XSOS_DATABASE_URL" | "XSOS_AUTHORIZATION_KEY"
        )));
    }
    #[test]
    fn log_level_and_trusted_proxies_follow_prefixed_environment_precedence() {
        assert_eq!(Settings::default().log_level, "info");
        assert!(Settings::default().trusted_proxies.is_empty());
        let file = br#"{"log_level":"warn","trusted_proxies":["192.0.2.0/24"]}"#;
        let environment = xcss::config::read_environment(&ENVIRONMENT, |name| match name {
            "XSOS_LOG_LEVEL" => Some("debug".into()),
            "XSOS_TRUSTED_PROXIES" => Some(r#"["127.0.0.1/32"]"#.into()),
            _ => None,
        })
        .unwrap();
        let loaded = xcss::config::resolve_validated(
            &Settings::default(),
            Some(file),
            &environment,
            &[],
            validate_intrinsic,
        )
        .unwrap();
        assert_eq!(loaded.value.log_level, "debug");
        assert_eq!(loaded.value.trusted_proxies, ["127.0.0.1/32"]);
        for file in [
            br#"{"log_level":"private-invalid-filter"}"#.as_slice(),
            br#"{"trusted_proxies":["invalid"]}"#.as_slice(),
        ] {
            assert!(
                xcss::config::resolve_validated(
                    &Settings::default(),
                    Some(file),
                    &[],
                    &[],
                    validate_intrinsic
                )
                .is_err()
            );
        }
    }
    #[test]
    fn runtime_configuration_rejects_bootstrap_passwords() {
        let file = br#"{"bootstrap_admin_password":"never-persist-this"}"#;
        assert!(
            xcss::config::resolve_validated(
                &Settings::default(),
                Some(file),
                &[],
                &[],
                validate_intrinsic
            )
            .is_err()
        );
    }
    #[test]
    fn bind_precedence_is_cli_then_prefixed_environment_then_file() {
        let file = br#"{"bind":"127.0.0.1:19001"}"#;
        let environment = xcss::config::read_environment(&ENVIRONMENT, |name| {
            (name == "XSOS_BIND").then(|| "127.0.0.1:19002".into())
        })
        .unwrap();
        let loaded = xcss::config::resolve_validated(
            &Settings::default(),
            Some(file),
            &environment,
            &[Override::new("/bind", "127.0.0.1:19003")],
            validate_intrinsic,
        )
        .unwrap();
        assert_eq!(
            loaded.value.bind,
            "127.0.0.1:19003".parse::<SocketAddr>().unwrap()
        );
        let loaded = xcss::config::resolve_validated(
            &Settings::default(),
            Some(file),
            &environment,
            &[],
            validate_intrinsic,
        )
        .unwrap();
        assert_eq!(
            loaded.value.bind,
            "127.0.0.1:19002".parse::<SocketAddr>().unwrap()
        );
    }
}

/// An omitted default file is optional; explicit files and unreadable defaults fail closed.
pub fn selected_config_path(config: Option<&Path>) -> anyhow::Result<Option<PathBuf>> {
    match config {
        Some(path) => Ok(Some(normalize_config_path(path)?)),
        None => {
            let path = PathBuf::from("/etc/isarmg/xsos/config.json");
            existing_default_config(path)
        }
    }
}

#[cfg(test)]
mod initialization_cli_tests {
    use super::*;
    #[test]
    fn explicit_username_and_global_configuration_are_accepted() {
        for args in [
            vec![
                "init",
                "--username",
                "admin",
                "--config",
                "/tmp/config.json",
            ],
            vec![
                "--config",
                "/tmp/config.json",
                "init",
                "--username",
                "admin",
            ],
        ] {
            assert!(Cli::try_parse_from(std::iter::once("xsos").chain(args)).is_ok());
        }
        assert!(
            Cli::try_parse_from([
                "xsos",
                "admin",
                "reset-password",
                "--username",
                "admin",
                "--database-url",
                "sqlite:///tmp/xsos.sqlite3"
            ])
            .is_err()
        );
        assert!(Cli::try_parse_from(["xsos", "check"]).is_ok());
        assert!(
            Cli::try_parse_from([
                "xsos",
                "admin",
                "reset-password",
                "--username",
                "admin",
                "--data-dir",
                "/tmp/xsos"
            ])
            .is_ok()
        );
    }
}

fn existing_default_config(path: PathBuf) -> anyhow::Result<Option<PathBuf>> {
    match std::fs::symlink_metadata(&path) {
        Ok(_) => Ok(Some(path)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[cfg(all(test, unix))]
mod default_configuration_tests {
    use super::*;
    #[test]
    fn dangling_default_symlink_is_not_treated_as_an_absent_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.json");
        assert!(existing_default_config(path.clone()).unwrap().is_none());
        std::os::unix::fs::symlink(directory.path().join("missing.json"), &path).unwrap();
        let selected = existing_default_config(path).unwrap().unwrap();
        assert!(xcss::config::read_private_file(&selected).is_err());
    }
}

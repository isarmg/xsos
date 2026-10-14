use std::{
    collections::BTreeMap,
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Parser;
#[cfg(test)]
use xcss::config::Override;
use xcss::config::{ConfigSource, EnvMapping, EnvValueKind};

use crate::retention::RetentionConfig;
use crate::telemetry::TelemetryWriterConfig;
use xcss::admin_auth::AdministratorOriginMode;

#[derive(Debug, Parser)]
#[command(name = "xsos", version, about = "Host telemetry service")]
pub struct Cli {
    #[arg(long)]
    pub root: PathBuf,
}

#[derive(Clone)]
pub struct ValidatedConfig {
    pub root: PathBuf,
    pub bind: SocketAddr,
    pub database_url: String,
    pub administrator_origin: AdministratorOriginMode,
    pub telemetry: TelemetryWriterConfig,
    pub retention: RetentionConfig,
    pub static_dir: Option<PathBuf>,
    pub log_level: String,
    pub data_dir: PathBuf,
    pub sources: BTreeMap<String, ConfigSource>,
}

impl ValidatedConfig {
    pub fn from_root_sources(root: &Path, config: &Path) -> anyhow::Result<Self> {
        anyhow::ensure!(
            root.is_absolute()
                && root != Path::new("/")
                && root.components().all(|part| matches!(
                    part,
                    std::path::Component::RootDir | std::path::Component::Normal(_)
                )),
            "root must be a normalized absolute directory"
        );
        let defaults = Settings {
            data_dir: root.join("state"),
            ..Settings::default()
        };

        let config = normalize_config_path(config)?;
        let file = xcss::config::read_managed_file(
            &config,
            cfg!(debug_assertions) && crate::release_contract::SOURCE_REVISION == "unbound",
        )?;
        let environment = xcss::config::read_environment(ENVIRONMENT, |name| env::var(name).ok())?;
        let loaded = xcss::config::resolve_validated(
            &defaults,
            Some(&file),
            &environment,
            &[],
            |settings, source| {
                crate::config_definition::validate(
                    settings,
                    root,
                    crate::release_contract::SOURCE_REVISION != "unbound",
                    source,
                )
            },
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
            root: root.to_path_buf(),
            bind,
            database_url,
            administrator_origin: cookie_mode,
            telemetry,
            retention,
            static_dir,
            log_level: settings.log_level,
            data_dir,
            sources: loaded.sources,
        })
    }
}

use crate::config_definition::Settings;
#[cfg(test)]
use crate::config_definition::validate_intrinsic;

const ENVIRONMENT: &[EnvMapping<'static>] = &[
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

#[cfg(test)]
mod precedence_contract_tests {
    use super::*;
    #[test]
    fn a_higher_priority_override_cannot_hide_an_invalid_file_value() {
        let file = serde_json::to_vec(&serde_json::json!({"telemetry_queue_capacity":0})).unwrap();
        let error = xcss::config::resolve_validated(
            &Settings::default(),
            Some(&file),
            &[Override::new(
                "/telemetry_queue_capacity",
                serde_json::json!(16),
            )],
            &[],
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

/// Normalize the required configuration path before private reads and reporting.
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
        assert!(ValidatedConfig::from_root_sources(directory.path(), &missing).is_err());
    }
    #[test]
    fn defaults_and_environment_belong_to_this_server() {
        let settings = Settings::default();
        assert!(!settings.development);
        assert_eq!(settings.data_dir, PathBuf::from("/opt/isarmg/xsos/state"));
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
    fn coupled_environment_overrides_are_validated_as_one_source() {
        let environment = xcss::config::read_environment(ENVIRONMENT, |name| match name {
            "XSOS_TELEMETRY_QUEUE_CAPACITY" => Some("32".into()),
            "XSOS_TELEMETRY_BATCH_SIZE" => Some("16".into()),
            _ => None,
        })
        .unwrap();
        let loaded = xcss::config::resolve_validated(
            &Settings::default(),
            Some(b"{}"),
            &environment,
            &[],
            validate_intrinsic,
        )
        .unwrap();
        assert_eq!(loaded.value.telemetry_queue_capacity, 32);
        assert_eq!(loaded.value.telemetry_batch_size, 16);
        assert_eq!(
            loaded.sources["/telemetry_queue_capacity"],
            ConfigSource::Environment
        );
        assert_eq!(
            loaded.sources["/telemetry_batch_size"],
            ConfigSource::Environment
        );
    }

    #[test]
    fn prefixed_environment_overrides_the_required_file() {
        let file = br#"{"bind":"127.0.0.1:19001"}"#;
        let environment = xcss::config::read_environment(ENVIRONMENT, |name| {
            (name == "XSOS_BIND").then(|| "127.0.0.1:19002".into())
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
        assert_eq!(
            loaded.value.bind,
            "127.0.0.1:19002".parse::<SocketAddr>().unwrap()
        );
    }
}

#[cfg(test)]
mod runtime_cli_tests {
    use super::*;
    #[test]
    fn server_has_only_root_based_runtime_arguments() {
        assert!(Cli::try_parse_from(["xsos", "--root", "/srv/xsos"]).is_ok());
        for removed in [
            "init",
            "check",
            "status",
            "config",
            "admin",
            "identity",
            "verify-release",
            "run",
        ] {
            assert!(Cli::try_parse_from(["xsos", removed]).is_err());
        }
    }
}

#[cfg(all(test, unix))]
mod required_configuration_tests {
    use super::*;
    #[test]
    fn a_linked_required_configuration_never_falls_back_to_defaults() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.json");
        std::os::unix::fs::symlink(directory.path().join("missing.json"), &path).unwrap();
        assert!(ValidatedConfig::from_root_sources(directory.path(), &path).is_err());
    }
}

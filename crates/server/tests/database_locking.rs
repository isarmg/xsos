#![cfg(target_os = "linux")]

use std::{
    ffi::OsString,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use chrono::Utc;
use uuid::Uuid;
use xsos::{
    database_lock::{ApplicationLock, MaintenanceLock},
    store,
};

const BOOTSTRAP_PASSWORD: &str = "lock-test-bootstrap-password";

fn database_url(path: &Path) -> String {
    format!("sqlite://{}", path.display())
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_owned();
    name.push(suffix);
    name.into()
}

fn server_command() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_xsos"));
    command.env_remove("RUST_LOG");
    command
}

fn configured_command(root: &Path, directory: &Path) -> Command {
    let mut command = server_command();
    command
        .env_clear()
        .arg("--root")
        .arg(root)
        .current_dir(directory);
    command
}

async fn close_workers(pool: &sqlx::SqlitePool) {
    let mut connections = Vec::new();
    for _ in 0..pool.size() {
        connections.push(pool.acquire().await.expect("own the SQLite worker"));
    }
    let closed = pool.close();
    tokio::pin!(closed);
    assert!(futures_util::poll!(&mut closed).is_pending());
    for connection in connections {
        connection.close().await.expect("stop the SQLite worker");
    }
    closed.await;
}

#[tokio::test]
async fn lock_identity_survives_working_directory_and_sqlite_restarts() {
    let temporary = tempfile::tempdir().expect("create test root");
    let root = temporary.path().join("root");
    std::fs::create_dir(&root).unwrap();
    for name in ["state", "config", "logs", "run"] {
        std::fs::create_dir(root.join(name)).unwrap();
        std::fs::set_permissions(root.join(name), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let directory = root.join("state");
    let config = root.join("config/config.json");
    std::fs::write(&config, br#"{"bind":"127.0.0.1:0","development":true}"#).unwrap();
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600)).unwrap();
    let database = directory.join("xsos.sqlite3");
    let url = database_url(&database);

    // Current CLI configuration has an absolute database authority, independent
    // of the child process working directory. Both lock protocols cover it.
    xsos::secrets::initialize(&directory).unwrap();
    let state_directory = xcss::state_file::PrivateStateDirectory::open(&directory).unwrap();
    let common = state_directory.try_instance_lock().unwrap();
    let application = ApplicationLock::acquire(&url).expect("acquire application lock");
    assert!(ApplicationLock::acquire(&url).is_err());
    let online = MaintenanceLock::shared(&url).expect("online maintenance shares the lock");
    assert!(
        MaintenanceLock::exclusive(&url).is_err(),
        "offline maintenance entered while the application was live"
    );

    let pool = store::open_or_initialize(&application.database_url())
        .await
        .expect("open SQLite through the trusted directory descriptor");
    store::ensure_admin_user(&pool, "admin", Some(BOOTSTRAP_PASSWORD))
        .await
        .expect("explicitly initialize the administrator fixture");
    let host_id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO monitored_hosts(\
           host_id,name,os,arch,client_version,registered_at,last_seen_at\
         ) VALUES(?,?,?,?,?,?,?)",
    )
    .bind(host_id)
    .bind("locked host")
    .bind("linux")
    .bind("x86_64")
    .bind(env!("CARGO_PKG_VERSION"))
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("persist through trusted descriptor URL");
    assert!(database.is_file());
    assert!(
        sidecar(&database, "-wal").is_file(),
        "WAL was not created beside the configured database"
    );
    assert!(
        sidecar(&database, "-shm").is_file(),
        "SHM was not created beside the configured database"
    );

    let validation = xsos::offline::validate_config(&root, &config)
        .await
        .expect("validate current configuration without a runtime maintenance command");
    assert_eq!(validation.data_dir, directory);
    assert!(validation.state_paths.contains(&directory));
    assert_eq!(
        validation.schema_identity,
        xsos::database_schema::expected_identity().unwrap()
    );

    let second_server = configured_command(&root, &directory)
        .output()
        .expect("run a second server against the same absolute database");
    assert!(!second_server.status.success());
    assert!(
        String::from_utf8_lossy(&second_server.stderr).contains("lock"),
        "second runtime did not reject the held lock: {}",
        String::from_utf8_lossy(&second_server.stderr)
    );
    for bytes in [&second_server.stdout, &second_server.stderr] {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains(BOOTSTRAP_PASSWORD));
        assert!(!text.contains(&url));
    }

    close_workers(&pool).await;
    // Capture only after this process releases its original SQLite descriptors.
    xsos::offline::check(&root, &config)
        .await
        .expect("read a validation snapshot without changing persisted state");
    drop(online);
    drop(application);
    drop(common);

    let offline = MaintenanceLock::exclusive(&url).expect("lock released after shutdown");
    assert!(ApplicationLock::acquire(&url).is_err());
    drop(offline);

    let restarted = ApplicationLock::acquire(&url).expect("reacquire lock after restart");
    xsos::database_schema::validate_configuration_database(&url)
        .expect("validate the physical database before SQLite opens it");
    let reopened = xsos::database_schema::open_validated_location(&restarted.database_url())
        .await
        .expect("reopen SQLite through a new trusted directory descriptor");
    let stored_name: String =
        sqlx::query_scalar("SELECT name FROM monitored_hosts WHERE host_id=?")
            .bind(host_id)
            .fetch_one(&reopened)
            .await
            .expect("read persisted data after restart");
    assert_eq!(stored_name, "locked host");
    close_workers(&reopened).await;
}

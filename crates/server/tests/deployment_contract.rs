#![cfg(target_os = "linux")]

use std::{fs, os::unix::fs::PermissionsExt, path::Path};

#[test]
fn follows_the_shared_deployment_contract() {
    let temporary = tempfile::tempdir().unwrap();
    fs::set_permissions(temporary.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let data_dir = temporary.path().join("instance %#?");
    let config = temporary.path().join("config.json");
    fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!({
            "data_dir": data_dir,
            "bind": "127.0.0.1:0",
            "development": true,
        }))
        .unwrap(),
    )
    .unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    xcss::testkit::assert_deployment_cli_contract(
        Path::new(env!("CARGO_BIN_EXE_xsos")),
        &config,
        &data_dir,
        &["xsos.sqlite3", "secrets.json"],
    );
}

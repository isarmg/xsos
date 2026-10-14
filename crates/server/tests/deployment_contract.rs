#![cfg(target_os = "linux")]
use std::{fs, os::unix::fs::PermissionsExt};
#[tokio::test]
async fn configuration_preserves_external_instance_data_and_fixed_root_log_paths() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("root");
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("config")).unwrap();
    fs::set_permissions(root.join("config"), fs::Permissions::from_mode(0o700)).unwrap();
    let data = temporary.path().join("external-instance-data");
    let config = root.join("config/config.json");
    fs::write(
        &config,
        serde_json::to_vec(
            &serde_json::json!({"data_dir":data,"bind":"127.0.0.1:0","development":true}),
        )
        .unwrap(),
    )
    .unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let value = xsos::config::ValidatedConfig::from_root_sources(&root, &config).unwrap();
    assert_eq!(value.root, root);
    assert_eq!(value.data_dir, data);
    assert!(!data.exists());
    assert!(!root.join("logs").exists());
    assert!(!root.join("run").exists());
    for protected in [
        "config",
        "bin",
        "libexec",
        "logs",
        "run",
        "share",
        "maintenance",
    ] {
        fs::write(&config,serde_json::to_vec(&serde_json::json!({"data_dir":root.join(protected),"bind":"127.0.0.1:0","development":true})).unwrap()).unwrap();
        assert!(xsos::config::ValidatedConfig::from_root_sources(&root, &config).is_err());
    }
}

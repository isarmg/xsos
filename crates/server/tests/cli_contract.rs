use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};
const BINARY: &str = env!("CARGO_BIN_EXE_xsos");
const SERVICE: &str = "xsos";
fn private(path: &Path) {
    fs::create_dir(path).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}
fn output(config: &Path, args: &[&str]) -> Output {
    use std::io::Write as _;
    let mut child = Command::new(BINARY)
        .env_clear()
        .args(args)
        .arg("--config")
        .arg(config)
        .arg("--json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if args.first() == Some(&"init") || args.starts_with(&["admin", "reset-password"]) {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"test-password1!\n")
            .unwrap();
    }
    child.wait_with_output().unwrap()
}
fn report(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "stdout is not one JSON record: {} / {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}
fn state(path: &Path) -> BTreeMap<PathBuf, (Vec<u8>, std::time::SystemTime)> {
    fn collect(
        root: &Path,
        path: &Path,
        result: &mut BTreeMap<PathBuf, (Vec<u8>, std::time::SystemTime)>,
    ) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            if meta.is_dir() {
                collect(root, &entry.path(), result)
            } else {
                result.insert(
                    entry.path().strip_prefix(root).unwrap().to_owned(),
                    (fs::read(entry.path()).unwrap(), meta.modified().unwrap()),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    if path.exists() {
        collect(path, path, &mut result);
    }
    result
}
struct Running(Child);
impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
#[test]
fn explicit_initialization_read_only_diagnostics_and_real_readiness() {
    let temporary = tempfile::tempdir().unwrap();
    fs::set_permissions(temporary.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let data = temporary.path().join("data %#?");
    let shared = temporary.path().join("shared");
    private(&shared);
    let socket = TcpListener::bind("127.0.0.1:0").unwrap();
    let bind = socket.local_addr().unwrap();
    drop(socket);
    let config = temporary.path().join("config.json");
    let mut settings = json!({"data_dir":data,"bind":bind.to_string(),"development":true});
    fs::write(&config, serde_json::to_vec(&settings).unwrap()).unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let empty = state(&data);
    for flag in ["--help", "--version"] {
        let result = Command::new(BINARY).env_clear().arg(flag).output().unwrap();
        assert!(result.status.success());
        assert_eq!(state(&data), empty);
    }
    for machine in [false, true] {
        let mut command = Command::new(BINARY);
        command
            .env_clear()
            .args(["run", "--bind", "private-cli-secret"]);
        if machine {
            command.arg("--json");
        }
        let rejected = command.output().unwrap();
        assert_eq!(rejected.status.code(), Some(2));
        assert!(!String::from_utf8_lossy(&rejected.stdout).contains("private-cli-secret"));
        assert!(!String::from_utf8_lossy(&rejected.stderr).contains("private-cli-secret"));
        if machine {
            assert_eq!(report(&rejected)["code"], "invalid_cli_input");
        }
    }
    let before_init = output(&config, &["config", "validate"]);
    assert!(
        before_init.status.success(),
        "{}",
        String::from_utf8_lossy(&before_init.stdout)
    );
    assert_eq!(report(&before_init)["status"], "valid");
    assert!(
        !data.exists(),
        "pure configuration validation created state"
    );
    let unchecked = output(&config, &["check"]);
    assert!(!unchecked.status.success());
    assert!(!data.exists(), "check initialized a missing instance");
    let missing = output(&config, &["run"]);
    assert!(!missing.status.success());
    assert!(report(&missing)["code"].is_string());
    assert_eq!(state(&data), empty, "run initialized an empty directory");
    assert!(!data.exists(), "run created a missing data directory");
    let invalid = output(&config, &["nonsense"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(report(&invalid)["code"].is_string());
    assert_eq!(state(&data), empty);
    let initialized = output(&config, &["init"]);
    assert!(
        initialized.status.success(),
        "{}",
        format!(
            "stdout={} stderr={}",
            String::from_utf8_lossy(&initialized.stdout),
            String::from_utf8_lossy(&initialized.stderr)
        )
    );
    assert_eq!(report(&initialized)["ready"], false);
    assert!(data.is_dir(), "init must create a missing data directory");
    let persisted_key = fs::read(data.join("secrets.json")).unwrap();
    assert_eq!(
        fs::metadata(data.join("secrets.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let secrets: Value = serde_json::from_slice(&persisted_key).unwrap();
    assert!(secrets["client_authorization_key"].is_string());
    assert!(
        !String::from_utf8_lossy(&initialized.stdout)
            .contains(secrets["client_authorization_key"].as_str().unwrap())
    );
    assert!(data.join("xsos.sqlite3").is_file());
    let initialized_state = state(&data);
    assert!(!initialized_state.is_empty());
    let again = output(&config, &["init"]);
    assert!(!again.status.success());
    report(&again);
    assert_eq!(
        state(&data),
        initialized_state,
        "repeated init changed current data"
    );
    let parent_traversal = output(
        &temporary.path().join("data/../config.json"),
        &["config", "validate"],
    );
    assert!(!parent_traversal.status.success());
    report(&parent_traversal);
    assert_eq!(state(&data), initialized_state);
    let valid = output(&config, &["config", "validate"]);
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    let valid_report = report(&valid);
    assert!(valid_report["schema_identity"].is_object());
    assert!(
        valid_report["state_paths"]
            .as_array()
            .unwrap()
            .iter()
            .all(|path| Path::new(path.as_str().unwrap()).is_absolute())
    );
    assert_eq!(
        state(&data),
        initialized_state,
        "validation changed bytes or timestamps"
    );
    let checked = output(&config, &["check"]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stdout)
    );
    assert_eq!(report(&checked)["initialized"], true);
    assert_eq!(report(&checked)["ready"], false);
    assert_eq!(
        state(&data),
        initialized_state,
        "check changed bytes or timestamps"
    );
    let stopped = output(&config, &["status"]);
    assert!(!stopped.status.success());
    report(&stopped);
    assert_eq!(
        state(&data),
        initialized_state,
        "offline status changed state"
    );
    let pending = data.join(".state-maintenance-pending.json");
    fs::write(&pending, b"{}").unwrap();
    fs::set_permissions(&pending, fs::Permissions::from_mode(0o600)).unwrap();
    let pending_state = state(&data);
    for command in ["run", "init", "check"] {
        let rejected = output(&config, &[command]);
        assert!(!rejected.status.success());
        assert_eq!(report(&rejected)["code"], "state.maintenance_pending");
        assert_eq!(state(&data), pending_state);
    }
    assert!(output(&config, &["config", "validate"]).status.success());
    assert_eq!(state(&data), pending_state);
    fs::remove_file(&pending).unwrap();
    let original = fs::read(&config).unwrap();
    settings["unexpected_private_field"] = json!("never-emit-this-secret");
    fs::write(&config, serde_json::to_vec(&settings).unwrap()).unwrap();
    let unknown = output(&config, &["config", "validate"]);
    assert!(!unknown.status.success());
    report(&unknown);
    assert!(!String::from_utf8_lossy(&unknown.stdout).contains("never-emit-this-secret"));
    assert_eq!(state(&data), initialized_state);
    fs::write(&config, original).unwrap();
    let child = Command::new(BINARY)
        .env_clear()
        .arg("run")
        .arg("--config")
        .arg(&config)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut running = Running(child);
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let status = output(&config, &["status"]);
        if status.status.success() {
            assert_eq!(report(&status)["service"], SERVICE);
            assert_eq!(report(&status)["ready"], true);
            break;
        }
        assert!(
            running.0.try_wait().unwrap().is_none(),
            "server exited before readiness: {}",
            String::from_utf8_lossy(&status.stdout)
        );
        assert!(
            Instant::now() < deadline,
            "service readiness deadline exceeded"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    let logged_request =
        assert_http_error(bind, "GET", "/api/v1/nonexistent-contract-route", "", 404);
    assert_http_error(bind, "POST", "/api/v1/auth/login", "{", 400);
    assert_http_error(bind, "GET", "/api/v1/auth/login", "", 405);
    assert_http_error(bind, "POST", "/api/v1/auth/login", &"x".repeat(70_000), 413);
    let log_deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let correlated = fs::read_dir(data.join("logs"))
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| {
                fs::read_to_string(entry.path())
                    .unwrap_or_default()
                    .lines()
                    .any(|line| {
                        serde_json::from_str::<Value>(line)
                            .ok()
                            .is_some_and(|record| {
                                record["request_id"] == logged_request
                                    && record["service"] == SERVICE
                            })
                    })
            });
        if correlated {
            break;
        }
        assert!(
            Instant::now() < log_deadline,
            "HTTP log did not correlate with response request ID"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let checked_live = output(&config, &["check"]);
    assert!(!checked_live.status.success());
    assert_eq!(report(&checked_live)["code"], "state.lock_busy");
    let while_running = output(&config, &["config", "validate"]);
    assert!(
        while_running.status.success(),
        "{}",
        String::from_utf8_lossy(&while_running.stdout)
    );
    let second_socket = TcpListener::bind("127.0.0.1:0").unwrap();
    let second_bind = second_socket.local_addr().unwrap();
    drop(second_socket);
    let second = output(&config, &["run", "--bind", &second_bind.to_string()]);
    assert!(!second.status.success());
    report(&second);
    assert!(output(&config, &["status"]).status.success());
    assert!(data.join("logs").is_dir());
    assert!(
        Command::new("kill")
            .args(["-TERM", &running.0.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    assert!(running.0.wait().unwrap().success());
    let records = fs::read_dir(data.join("logs"))
        .unwrap()
        .filter_map(Result::ok)
        .flat_map(|entry| {
            fs::read_to_string(entry.path())
                .unwrap_or_default()
                .lines()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .filter_map(|line| serde_json::from_str::<Value>(&line).ok())
        .collect::<Vec<_>>();
    for event in [
        "common.config.loaded",
        "common.runtime.started",
        "common.runtime.shutdown_started",
        "common.runtime.stopped",
    ] {
        assert!(
            records
                .iter()
                .any(|record| record["event"] == event && record["service"] == SERVICE),
            "missing common event {event}: {records:?}"
        );
    }
    assert_eq!(fs::read(data.join("secrets.json")).unwrap(), persisted_key);
    let restarted = Command::new(BINARY)
        .env_clear()
        .args(["run", "--config"])
        .arg(&config)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut restarted = Running(restarted);
    let deadline = Instant::now() + Duration::from_secs(15);
    while !output(&config, &["status"]).status.success() {
        assert!(
            restarted.0.try_wait().unwrap().is_none(),
            "restarted server exited"
        );
        assert!(
            Instant::now() < deadline,
            "restarted service never became ready"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(fs::read(data.join("secrets.json")).unwrap(), persisted_key);
    assert!(
        Command::new("kill")
            .args(["-TERM", &restarted.0.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    assert!(restarted.0.wait().unwrap().success());
    let reset = output(&config, &["admin", "reset-password", "--username", "admin"]);
    assert!(
        reset.status.success(),
        "{}",
        String::from_utf8_lossy(&reset.stdout)
    );
    assert_eq!(report(&reset)["status"], "password-reset");
    assert_eq!(fs::read(data.join("secrets.json")).unwrap(), persisted_key);
}

fn assert_http_error(
    bind: std::net::SocketAddr,
    method: &str,
    path: &str,
    body: &str,
    status: u16,
) -> String {
    use std::io::{Read, Write};
    let mut connection = std::net::TcpStream::connect(bind).unwrap();
    connection
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    write!(connection, "{method} {path} HTTP/1.1\r\nHost: {bind}\r\nOrigin: http://{bind}\r\nSec-Fetch-Site: same-origin\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
    let mut response = Vec::new();
    connection.read_to_end(&mut response).unwrap();
    let boundary = response
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .unwrap();
    let head = String::from_utf8_lossy(&response[..boundary]);
    assert!(
        head.lines()
            .next()
            .unwrap()
            .contains(&format!(" {status} ")),
        "{head}"
    );
    let headers = head
        .lines()
        .skip(1)
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        headers.get("cache-control").map(String::as_str),
        Some("no-store")
    );
    let request_id = headers
        .get("x-request-id")
        .expect("error response has no request ID");
    let mut payload = response[boundary + 4..].to_vec();
    if headers
        .get("transfer-encoding")
        .is_some_and(|value| value == "chunked")
    {
        let mut decoded = Vec::new();
        let mut cursor = 0;
        loop {
            let end = payload[cursor..]
                .windows(2)
                .position(|bytes| bytes == b"\r\n")
                .unwrap()
                + cursor;
            let length = usize::from_str_radix(
                std::str::from_utf8(&payload[cursor..end])
                    .unwrap()
                    .split(';')
                    .next()
                    .unwrap(),
                16,
            )
            .unwrap();
            if length == 0 {
                break;
            }
            cursor = end + 2;
            decoded.extend_from_slice(&payload[cursor..cursor + length]);
            cursor += length + 2;
        }
        payload = decoded;
    }
    let record: Value = serde_json::from_slice(&payload).unwrap_or_else(|_| {
        panic!(
            "non JSON error: {} / {head}",
            String::from_utf8_lossy(&payload)
        )
    });
    assert!(record["code"].is_string(), "{record}");
    assert_eq!(record["request_id"].as_str(), Some(request_id.as_str()));
    request_id.clone()
}

#[test]
fn missing_or_corrupt_persistent_keys_are_never_repaired_by_run_or_check() {
    let temporary = tempfile::tempdir().unwrap();
    fs::set_permissions(temporary.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let data = temporary.path().join("instance");
    let config = temporary.path().join("config.json");
    fs::write(
        &config,
        serde_json::to_vec(&json!({"data_dir": data, "bind":"127.0.0.1:0", "development":true}))
            .unwrap(),
    )
    .unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    let initialized = output(&config, &["init"]);
    assert!(
        initialized.status.success(),
        "{} / {}",
        String::from_utf8_lossy(&initialized.stdout),
        String::from_utf8_lossy(&initialized.stderr)
    );
    let secrets_path = data.join("secrets.json");
    let valid_keys = fs::read(&secrets_path).unwrap();
    fs::remove_file(&secrets_path).unwrap();
    for invalid in [None, Some(b"not JSON".as_slice()), Some(br#"{"client_authorization_key":"invalid"}"#.as_slice()), Some(br#"{"client_authorization_key":"BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc=","unexpected":true}"#.as_slice())] {
        if let Some(bytes) = invalid {
            fs::write(&secrets_path, bytes).unwrap();
            fs::set_permissions(&secrets_path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let before = state(&data);
        assert!(output(&config, &["config", "validate"]).status.success());
        for command in ["run", "check"] {
            let rejected = output(&config, &[command]);
            assert!(!rejected.status.success(), "{command} accepted missing or corrupt persisted keys");
            report(&rejected);
            assert_eq!(state(&data), before, "{command} repaired or changed instance state");
        }
    }
    fs::write(&secrets_path, valid_keys).unwrap();
    assert!(output(&config, &["check"]).status.success());
}

#[test]
fn offline_binary_artifacts_do_not_read_instance_configuration() {
    let inaccessible = Path::new("/missing-instance/../unused-config.json");
    for command in ["identity", "web-assets"] {
        let result = output(inaccessible, &[command]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

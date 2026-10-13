#[path = "../../scripts/server-target-gate.rs"]
mod target_gate;
use std::{collections::BTreeSet, env, fs};

const XCSS_SOURCE: &str = "git+https://github.com/isarmg/xcss.git?rev=";

fn locked_xcss_revision(lockfile: &str) -> String {
    let revisions = lockfile
        .lines()
        .map(str::trim)
        .filter_map(|line| line.strip_prefix("source = \"")?.strip_suffix('"'))
        .filter_map(|source| source.strip_prefix(XCSS_SOURCE))
        .map(|source| source.split_once('#').expect("locked xcss source"))
        .map(|(requested, locked)| {
            assert_eq!(requested, locked, "xcss revision must be immutable");
            assert!(
                locked.len() == 40 && locked.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "xcss revision must be full hexadecimal"
            );
            locked.to_owned()
        })
        .collect::<BTreeSet<_>>();
    if revisions.is_empty() {
        assert_eq!(
            env::var("PROFILE").as_deref(),
            Ok("debug"),
            "release builds require a source-bound xcss dependency"
        );
        assert!(
            env::var_os("XSSI_BUILD_METADATA_DIR").is_none(),
            "release metadata cannot describe an unbound foundation"
        );
        return "unbound".into();
    }
    assert_eq!(
        revisions.len(),
        1,
        "the xcss package must use one immutable revision"
    );
    revisions.into_iter().next().expect("one xcss revision")
}

fn main() {
    println!("cargo:rerun-if-env-changed=XSSI_BUILD_METADATA_DIR");
    let target = env::var("TARGET").expect("Cargo must provide TARGET");
    target_gate::main();
    let product_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("server crate belongs to the product workspace");
    let web_root = env::var_os("XCSS_WEB_DIST")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| product_root.join("web/dist"));
    xcss::web_assets::build::generate(&web_root).expect("generate embedded Web assets");
    println!("cargo:rerun-if-env-changed=XCSS_WEB_DIST");
    let source_revision = env::var("XSOS_SOURCE_REVISION").unwrap_or_else(|_| "unbound".to_owned());
    assert!(
        source_revision == "unbound"
            || (source_revision.len() == 40
                && source_revision
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))),
        "XSOS_SOURCE_REVISION must be a full lowercase 40-hex Git commit"
    );
    println!("cargo:rustc-env=XSOS_BUILD_TARGET={target}");
    println!("cargo:rustc-env=XSOS_SOURCE_REVISION={source_revision}");
    let xcss_revision = locked_xcss_revision(
        &fs::read_to_string(product_root.join("Cargo.lock")).expect("read Cargo.lock"),
    );
    println!("cargo:rustc-env=XCSS_REVISION={xcss_revision}");
    println!("cargo:rerun-if-env-changed=XSOS_SOURCE_REVISION");
    println!(
        "cargo:rerun-if-changed={}",
        product_root.join("Cargo.lock").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        product_root.join("release.json").display()
    );
    emit_offline_metadata();
}

fn emit_offline_metadata() {
    use sha2::{Digest, Sha256};
    use xcss::contracts::{StateContract, StateResource, StateResourceKind, StateSchemaIdentity};
    let Some(destination) = std::env::var_os("XSSI_BUILD_METADATA_DIR") else {
        return;
    };
    let destination = std::path::PathBuf::from(destination);
    assert!(
        destination.is_absolute() && destination.is_dir(),
        "explicit metadata destination must exist and be absolute"
    );
    let revision = std::env::var("XSOS_SOURCE_REVISION")
        .expect("release metadata requires a source-bound build");
    let hash = "cb768892031e80900b95395aae25397fac4c37f3aac8b912d911d65c6a277afe";
    let schema =
        xcss::contracts::SchemaIdentity::new("xsos", "1.0.0", 1, hash).expect("schema identity");
    let contract = StateContract {
        contract_version: xcss::contracts::STATE_CONTRACT_VERSION,
        application: "xsos".into(),
        application_version: std::env::var("CARGO_PKG_VERSION").unwrap(),
        source_revision: revision.clone(),
        schema: Some(StateSchemaIdentity {
            revision: 1,
            sha256: hash.into(),
        }),
        maintenance_locks: vec![
            ".state-instance.lock".into(),
            ".state-maintenance.lock".into(),
        ],
        resources: vec![
            StateResource {
                name: "state".into(),
                kind: StateResourceKind::DataTree,
                required: true,
            },
            StateResource {
                name: "config".into(),
                kind: StateResourceKind::Configuration,
                required: true,
            },
        ],
        external_requirements: vec![],
        companion_contracts: vec![],
    };
    contract.validate().expect("current state contract");
    let bytes = serde_json::to_vec(&contract).unwrap();
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let identity = xcss::contracts::ReleaseIdentity {
        product: "xsos".into(),
        version: std::env::var("CARGO_PKG_VERSION").unwrap(),
        source_revision: revision,
        target: xcss::server_target::SERVER_TARGET_TRIPLE.into(),
        state_contract_sha256: digest,
    };
    identity.validate().expect("release identity");
    std::fs::write(
        destination.join("xsos-identity.json"),
        serde_json::to_vec(&identity).unwrap(),
    )
    .unwrap();
    std::fs::write(
        destination.join("xsos-schema.json"),
        serde_json::to_vec(&schema).unwrap(),
    )
    .unwrap();
    std::fs::write(destination.join("xsos-state-contract.json"), bytes).unwrap();
    let generated = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap())
        .join("xcss-web-assets.rs.manifest.json");
    std::fs::copy(generated, destination.join("xsos-web-assets.json"))
        .expect("copy exact embedded Web manifest");
}

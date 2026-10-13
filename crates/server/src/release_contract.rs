use xcss::server_target::SERVER_TARGET_TRIPLE;
pub const SUPPORTED_SERVER_TARGET: &str = SERVER_TARGET_TRIPLE;
pub const BUILD_TARGET: &str = env!("XSOS_BUILD_TARGET");
pub const SOURCE_REVISION: &str = env!("XSOS_SOURCE_REVISION");
pub fn ensure_supported_runtime() -> anyhow::Result<()> {
    anyhow::ensure!(
        BUILD_TARGET == SUPPORTED_SERVER_TARGET
            && cfg!(all(
                target_arch = "x86_64",
                target_os = "linux",
                target_env = "gnu"
            )),
        "xsos binary target must be {SUPPORTED_SERVER_TARGET}"
    );

    let runtime = rustix::system::uname();
    validate_runtime_platform(runtime.sysname().to_bytes(), runtime.machine().to_bytes())
}

fn validate_runtime_platform(sysname: &[u8], machine: &[u8]) -> anyhow::Result<()> {
    anyhow::ensure!(
        sysname == b"Linux" && machine == b"x86_64",
        "xsos only runs on x86_64 GNU/Linux; runtime reported system={:?}, machine={:?}",
        String::from_utf8_lossy(sysname),
        String::from_utf8_lossy(machine)
    );
    Ok(())
}

//! Private application-state fixture; does not test installation or setup initialization.
#[path = "../tests/support/database.rs"]
mod database;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let root = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("fixture ROOT required"))?,
    );
    database::initialize_root(&root).await
}

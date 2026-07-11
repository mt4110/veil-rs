#[tokio::main]
async fn main() -> anyhow::Result<()> {
    veil_lsp::server::run_stdio().await
}

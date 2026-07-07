#[tokio::main]
async fn main() {
    veil_lsp::server::run_stdio().await;
}

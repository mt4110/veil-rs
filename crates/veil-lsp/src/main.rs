#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = match std::env::var("VEIL_LSP_CONFIG_JSON") {
        Ok(value) => serde_json::from_str::<veil_config::Config>(&value)?,
        Err(std::env::VarError::NotPresent) => veil_config::Config::default(),
        Err(error) => return Err(error.into()),
    };
    veil_lsp::server::run_stdio_with_config(config).await
}

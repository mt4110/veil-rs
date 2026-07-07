use std::process::ExitCode;

fn main() -> ExitCode {
    eprintln!(
        "{} scaffold is present; tower-lsp integration is not implemented yet.",
        veil_lsp::server::SERVER_NAME
    );
    ExitCode::from(2)
}

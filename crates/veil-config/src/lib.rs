pub mod config;
pub mod loader;
pub mod validate;

pub use config::{Config, MaskMode, OutputConfig, RuleConfig};
pub use loader::load_config;

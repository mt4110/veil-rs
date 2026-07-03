pub mod config;
pub mod loader;
pub mod presets;
pub mod validate;

pub use config::{Config, MaskMode, OutputConfig, RuleConfig};
pub use loader::load_config;
pub use presets::{
    apply_builtin_preset_as_base, builtin_preset_config, BUILTIN_PRESET_IDS,
    LOGS_JP_REQUIRED_RULE_IDS,
};

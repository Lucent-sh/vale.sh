use crate::error::{ValeError, ValeResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub core: CoreConfig,
    pub providers: ProvidersConfig,
    pub lean: LeanConfig,
    pub risk: RiskConfig,
    pub report: ReportConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreConfig {
    pub default_engine: String,
    pub default_output: String,
    pub cache_dir: String,
    pub parallelism: usize,
    pub color: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProvidersConfig {
    pub default: String,
    pub polygon: PolygonConfig,
    pub alpaca: AlpacaConfig,
    pub yahoo: YahooConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolygonConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlpacaConfig {
    pub api_key: String,
    pub secret_key: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YahooConfig {
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LeanConfig {
    pub executable: String,
    pub docker_image: String,
    pub python_env: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RiskConfig {
    pub default_var_confidence: f64,
    pub risk_free_rate: f64,
    pub annualization_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportConfig {
    pub default_format: String,
    pub html_open_browser: bool,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub theme: String,
    pub sparklines: bool,
    pub animations: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            default_engine: "native".into(),
            default_output: "table".into(),
            cache_dir: "~/.vale/cache".into(),
            parallelism: num_cpus::get(),
            color: true,
        }
    }
}

impl Default for ProvidersConfig {
    fn default() -> Self {
        Self {
            default: "yahoo".into(),
            polygon: Default::default(),
            alpaca: AlpacaConfig {
                base_url: "https://paper-api.alpaca.markets".into(),
                ..Default::default()
            },
            yahoo: YahooConfig { timeout_secs: 10 },
        }
    }
}

impl Default for LeanConfig {
    fn default() -> Self {
        Self {
            executable: "lean".into(),
            docker_image: "quantconnect/lean:latest".into(),
            python_env: String::new(),
        }
    }
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            default_var_confidence: 0.95,
            risk_free_rate: 0.05,
            annualization_factor: 252.0,
        }
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            default_format: "table".into(),
            html_open_browser: false,
            output_dir: "./vale-reports".into(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "auto".into(),
            sparklines: true,
            animations: true,
        }
    }
}

impl Config {
    /// Load config: global (~/.vale/config.toml) merged with project (./vale.toml).
    pub fn load() -> ValeResult<Self> {
        let mut config = Self::default();

        if let Some(home) = dirs::home_dir() {
            let global = home.join(".vale").join("config.toml");
            if global.exists() {
                let text = std::fs::read_to_string(&global)?;
                let parsed: Config =
                    toml::from_str(&text).map_err(|e| ValeError::Config(e.to_string()))?;
                config = parsed;
            }
        }

        let project = PathBuf::from("vale.toml");
        if project.exists() {
            let text = std::fs::read_to_string(&project)?;
            let parsed: Config =
                toml::from_str(&text).map_err(|e| ValeError::Config(e.to_string()))?;
            config = parsed;
        }

        Ok(config)
    }

    pub fn cache_dir(&self) -> PathBuf {
        PathBuf::from(shellexpand::tilde(&self.core.cache_dir).to_string())
    }

    pub fn init_global() -> ValeResult<()> {
        let home = dirs::home_dir()
            .ok_or_else(|| ValeError::Config("Cannot find home directory".into()))?;
        let vale_dir = home.join(".vale");
        std::fs::create_dir_all(&vale_dir)?;
        let config_path = vale_dir.join("config.toml");
        if !config_path.exists() {
            let default = toml::to_string_pretty(&Config::default())
                .map_err(|e| ValeError::Config(e.to_string()))?;
            std::fs::write(&config_path, default)?;
        }
        Ok(())
    }
}

use crate::adapter::AdapterStatus;
use std::path::PathBuf;
use std::process::Command;
use vale_core::config::Config;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DoctorReport {
    pub vale_version: String,
    pub config_path: Option<PathBuf>,
    pub cache_size_bytes: u64,
    pub data_providers: Vec<AdapterStatus>,
    pub backtest_engines: Vec<AdapterStatus>,
    pub portfolio_optimizers: Vec<AdapterStatus>,
    pub pricing_engines: Vec<AdapterStatus>,
}

impl DoctorReport {
    pub async fn run(config: &Config) -> Self {
        let config_path = dirs::home_dir()
            .map(|h| h.join(".vale").join("config.toml"))
            .filter(|p| p.exists());

        let cache_dir = config.cache_dir();
        let cache_size = dir_size(&cache_dir).unwrap_or(0);

        let mut data_providers = Vec::new();
        data_providers.push(check_yahoo().await);
        data_providers.push(AdapterStatus {
            name: "polygon".into(),
            available: !config.providers.polygon.api_key.is_empty(),
            version: None,
            location: None,
            message: if config.providers.polygon.api_key.is_empty() {
                Some("key not configured".into())
            } else {
                Some("key configured".into())
            },
        });
        data_providers.push(AdapterStatus {
            name: "alpaca".into(),
            available: !config.providers.alpaca.api_key.is_empty(),
            version: None,
            location: None,
            message: if config.providers.alpaca.api_key.is_empty() {
                Some("key not configured".into())
            } else {
                Some("key configured".into())
            },
        });

        let mut backtest_engines = vec![AdapterStatus {
            name: "native".into(),
            available: true,
            version: Some("built-in".into()),
            location: None,
            message: None,
        }];
        backtest_engines.push(check_command("lean", &["--version"]));
        backtest_engines.push(check_python(
            "vectorbt",
            "import vectorbt; print(vectorbt.__version__)",
        ));

        let mut portfolio_optimizers = vec![AdapterStatus {
            name: "native".into(),
            available: true,
            version: Some("built-in".into()),
            location: None,
            message: None,
        }];
        portfolio_optimizers.push(check_python(
            "skfolio",
            "import skfolio; print(skfolio.__version__)",
        ));
        portfolio_optimizers.push(check_python(
            "pypfopt",
            "import pypfopt; print(pypfopt.__version__)",
        ));

        let pricing_engines = vec![
            AdapterStatus {
                name: "black-scholes".into(),
                available: true,
                version: Some("native".into()),
                location: None,
                message: None,
            },
            check_python(
                "quantlib (pyql)",
                "import QuantLib; print(QuantLib.__version__)",
            ),
        ];

        Self {
            vale_version: env!("CARGO_PKG_VERSION").to_string(),
            config_path,
            cache_size_bytes: cache_size,
            data_providers,
            backtest_engines,
            portfolio_optimizers,
            pricing_engines,
        }
    }

    /// CSV rows: category,name,available,version,message
    pub fn to_csv(&self) -> String {
        let mut out =
            String::from("category,name,available,version,message\n");
        append_status_rows(&mut out, "core", &core_rows(self));
        append_status_rows(&mut out, "data", &self.data_providers);
        append_status_rows(&mut out, "backtest", &self.backtest_engines);
        append_status_rows(&mut out, "portfolio", &self.portfolio_optimizers);
        append_status_rows(&mut out, "pricing", &self.pricing_engines);
        out
    }
}

fn core_rows(report: &DoctorReport) -> Vec<AdapterStatus> {
    vec![
        AdapterStatus {
            name: "vale".into(),
            available: true,
            version: Some(report.vale_version.clone()),
            location: None,
            message: None,
        },
        AdapterStatus {
            name: "config".into(),
            available: report.config_path.is_some(),
            version: None,
            location: report.config_path.as_ref().map(|p| p.display().to_string()),
            message: None,
        },
        AdapterStatus {
            name: "cache".into(),
            available: report.cache_size_bytes > 0,
            version: None,
            location: None,
            message: Some(format!("{:.1} MB", report.cache_size_bytes as f64 / 1_000_000.0)),
        },
    ]
}

fn append_status_rows(out: &mut String, category: &str, rows: &[AdapterStatus]) {
    for s in rows {
        let ver = s.version.as_deref().unwrap_or("");
        let msg = s.message.as_deref().unwrap_or("").replace(',', ";");
        out.push_str(&format!(
            "{category},{},{},{ver},{msg}\n",
            csv_escape(&s.name),
            s.available
        ));
    }
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

async fn check_yahoo() -> AdapterStatus {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();
    let available = match client {
        Ok(c) => c
            .get("https://query1.finance.yahoo.com/v8/finance/chart/SPY?interval=1d&range=1d")
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false),
        Err(_) => false,
    };
    AdapterStatus {
        name: "yahoo".into(),
        available,
        version: None,
        location: None,
        message: if available {
            Some("reachable".into())
        } else {
            Some("not reachable".into())
        },
    }
}

fn check_command(name: &str, args: &[&str]) -> AdapterStatus {
    let output = Command::new(name).args(args).output();
    match output {
        Ok(out) if out.status.success() => AdapterStatus {
            name: name.into(),
            available: true,
            version: Some(String::from_utf8_lossy(&out.stdout).trim().to_string()),
            location: which_path(name),
            message: None,
        },
        _ => AdapterStatus {
            name: name.into(),
            available: false,
            version: None,
            location: None,
            message: Some(format!(
                "{name} is not installed. Run `vale doctor` to see installation instructions."
            )),
        },
    }
}

fn check_python(display: &str, code: &str) -> AdapterStatus {
    let output = Command::new("python3").args(["-c", code]).output();
    match output {
        Ok(out) if out.status.success() => AdapterStatus {
            name: display.into(),
            available: true,
            version: Some(String::from_utf8_lossy(&out.stdout).trim().to_string()),
            location: which_path("python3"),
            message: None,
        },
        _ => AdapterStatus {
            name: display.into(),
            available: false,
            version: None,
            location: None,
            message: Some(format!("not found — install: pip install {display}")),
        },
    }
}

fn which_path(cmd: &str) -> Option<String> {
    Command::new("which")
        .arg(cmd)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn dir_size(path: &PathBuf) -> std::io::Result<u64> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0u64;
    if path.is_file() {
        return Ok(path.metadata()?.len());
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            total += dir_size(&entry.path())?;
        } else {
            total += meta.len();
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn doctor_report_serializes() {
        let config = Config::default();
        let report = DoctorReport::run(&config).await;
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("vale_version"));
    }
}

use crate::theme;
use anyhow::Result;
use std::process::Command;
use vale_adapters::doctor::DoctorReport;
use vale_core::config::Config;
use vale_core::types::OutputFormat;

pub async fn handle(output: OutputFormat) -> Result<()> {
    let config = Config::load().unwrap_or_default();

    match output {
        OutputFormat::Json => {
            let report = DoctorReport::run(&config).await;
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(());
        }
        OutputFormat::Csv => {
            anyhow::bail!("CSV output not supported for doctor");
        }
        OutputFormat::Table => {}
    }

    theme::section_header("Core");
    theme::status_line("vale", env!("CARGO_PKG_VERSION"), true);

    let config_path = dirs::home_dir()
        .map(|h| h.join(".vale").join("config.toml"))
        .filter(|p| p.exists());
    theme::status_line(
        "config",
        &config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "not found — run: vale config init".into()),
        config_path.is_some(),
    );

    let cache_dir = config.cache_dir();
    let cache_size = dir_size(&cache_dir).unwrap_or(0);
    theme::status_line(
        "cache",
        &format!(
            "{} ({:.1} MB)",
            cache_dir.display(),
            cache_size as f64 / 1_000_000.0
        ),
        cache_dir.exists(),
    );

    println!();
    theme::section_header("Data Providers");
    check_yahoo();
    let polygon_ok = !config.providers.polygon.api_key.is_empty();
    theme::status_line(
        "polygon",
        if polygon_ok {
            "key configured"
        } else {
            "key not configured — run: vale config set providers.polygon.api_key <KEY>"
        },
        polygon_ok,
    );
    let alpaca_ok = !config.providers.alpaca.api_key.is_empty();
    theme::status_line(
        "alpaca",
        if alpaca_ok {
            "key configured"
        } else {
            "key not configured — run: vale config set providers.alpaca.api_key <KEY>"
        },
        alpaca_ok,
    );

    println!();
    theme::section_header("Backtest Engines");
    theme::status_line("native", "built-in", true);
    check_python_package("lean", "lean --version");
    check_python_package("vectorbt", "import vectorbt; print(vectorbt.__version__)");

    println!();
    theme::section_header("Portfolio Optimizers");
    theme::status_line(
        "native (equal_weight, min_variance, max_sharpe)",
        "built-in",
        true,
    );
    check_python_package("skfolio", "import skfolio; print(skfolio.__version__)");
    check_python_package("pypfopt", "import pypfopt; print(pypfopt.__version__)");

    println!();
    theme::section_header("Pricing Engines");
    theme::status_line("black-scholes", "built-in", true);
    check_python_package(
        "quantlib (pyql)",
        "import QuantLib; print(QuantLib.__version__)",
    );

    Ok(())
}

fn check_yahoo() {
    let result = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "https://query1.finance.yahoo.com",
        ])
        .output();
    let ok = matches!(result, Ok(ref o) if String::from_utf8_lossy(&o.stdout).trim() == "200");
    theme::status_line(
        "yahoo",
        if ok { "reachable" } else { "check connection" },
        ok,
    );
}

fn check_python_package(display_name: &str, code: &str) {
    let result = Command::new("python3").args(["-c", code]).output();
    match result {
        Ok(out) if out.status.success() => {
            let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
            theme::status_line(display_name, &ver, true);
        }
        _ => {
            theme::status_line(
                display_name,
                &format!("not found — install: pip install {display_name}"),
                false,
            );
        }
    }
}

fn dir_size(path: &std::path::Path) -> std::io::Result<u64> {
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

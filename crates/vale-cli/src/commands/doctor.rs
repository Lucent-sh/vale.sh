use crate::theme;
use anyhow::Result;
use vale_adapters::adapter::AdapterStatus;
use vale_adapters::doctor::DoctorReport;
use vale_core::config::Config;
use vale_core::types::OutputFormat;

pub async fn handle(output: OutputFormat) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let report = DoctorReport::run(&config).await;

    match output {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        OutputFormat::Csv => {
            print!("{}", report.to_csv());
        }
        OutputFormat::Table => {
            theme::section_header("Core");
            print_adapter_section(&[
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
                    available: true,
                    version: None,
                    location: None,
                    message: Some(format!(
                        "{:.1} MB",
                        report.cache_size_bytes as f64 / 1_000_000.0
                    )),
                },
            ]);

            print_adapter_section_named("Data Providers", &report.data_providers);
            print_adapter_section_named("Backtest Engines", &report.backtest_engines);
            print_adapter_section_named("Portfolio Optimizers", &report.portfolio_optimizers);
            print_adapter_section_named("Pricing Engines", &report.pricing_engines);
        }
    }
    Ok(())
}

fn print_adapter_section_named(title: &str, rows: &[AdapterStatus]) {
    println!();
    theme::section_header(title);
    print_adapter_section(rows);
}

fn print_adapter_section(rows: &[AdapterStatus]) {
    for s in rows {
        let detail = s
            .version
            .as_deref()
            .or(s.message.as_deref())
            .or(s.location.as_deref())
            .unwrap_or(if s.available { "ok" } else { "unavailable" });
        theme::status_line(&s.name, detail, s.available);
    }
}

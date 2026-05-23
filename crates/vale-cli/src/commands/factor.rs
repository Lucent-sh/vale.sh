use crate::cli::FactorCommand;
use crate::theme;
use anyhow::Result;
use vale_core::types::OutputFormat;
use vale_factor::ic::information_coefficient;
use vale_factor::regression::ols;
use vale_report::json::FactorReportJson;

pub async fn handle(cmd: FactorCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        FactorCommand::Analyze(args) => {
            let returns = load_returns_csv(&args.returns)?;
            let factors = load_factor_data(&args.model).await?;
            let n = returns.len().min(factors.mkt_rf.len());
            let y: Vec<f64> = returns[..n].to_vec();
            let x = vec![
                factors.mkt_rf[..n].to_vec(),
                factors.smb[..n].to_vec(),
                factors.hml[..n].to_vec(),
            ];
            let result = ols(&y, &x);
            let report = FactorReportJson {
                alpha: result.alpha,
                betas: result.betas,
                t_stats: result.t_stats,
                r_squared: result.r_squared,
                information_ratio: result.information_ratio,
            };
            match output {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                OutputFormat::Csv => {
                    println!("metric,value");
                    println!("alpha,{}", report.alpha);
                    println!("r_squared,{}", report.r_squared);
                }
                OutputFormat::Table => {
                    theme::section_header("Factor Analysis");
                    let mut table = vale_report::table::factor_table(&report);
                    theme::table_style(&mut table);
                    println!("{table}");
                }
            }
        }
        FactorCommand::Ic(args) => {
            let signals = load_single_column_csv(&args.signals)?;
            let returns = load_single_column_csv(&args.returns)?;
            let ics = information_coefficient(&signals, &returns, &args.periods);
            match output {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&ics)?),
                OutputFormat::Csv => {
                    println!("lag,ic");
                    for (lag, ic) in &ics {
                        println!("{lag},{ic}");
                    }
                }
                OutputFormat::Table => {
                    theme::section_header("Information Coefficient");
                    for (lag, ic) in &ics {
                        theme::status_line(
                            &format!("lag {lag}"),
                            &format!("{ic:.4}"),
                            ic.abs() > 0.05,
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

fn load_returns_csv(path: &std::path::Path) -> Result<Vec<f64>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut vals = Vec::new();
    for rec in rdr.records() {
        let r = rec?;
        if let Some(v) = r.get(1).and_then(|s| s.parse().ok()) {
            vals.push(v);
        }
    }
    Ok(vals)
}

fn load_single_column_csv(path: &std::path::Path) -> Result<Vec<f64>> {
    load_returns_csv(path)
}

async fn load_factor_data(model: &str) -> Result<vale_factor::fama_french::FactorData> {
    match model {
        "ff3" | "ff5" | "carhart4" => vale_factor::fama_french::load_ff3()
            .await
            .map_err(|e| anyhow::anyhow!("{e}")),
        other => anyhow::bail!("unknown factor model: {other}"),
    }
}

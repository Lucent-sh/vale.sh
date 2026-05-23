use crate::cli::StrategyCommand;
use crate::theme;
use anyhow::Result;

pub async fn handle(cmd: StrategyCommand) -> Result<()> {
    match cmd {
        StrategyCommand::Scaffold(args) => {
            let dir = args
                .output
                .clone()
                .unwrap_or_else(|| std::path::PathBuf::from(&args.name));
            std::fs::create_dir_all(&dir)?;
            match args.template.as_str() {
                "lean-python" => {
                    let main_py = format!(
                        r#"from AlgorithmImports import *

class {}Algorithm(QCAlgorithm):
    def Initialize(self):
        self.SetStartDate(2020, 1, 1)
        self.SetEndDate(2024, 1, 1)
        self.SetCash(100000)
        self.AddEquity("SPY", Resolution.Daily)

    def OnData(self, data):
        if not self.Portfolio.Invested:
            self.SetHoldings("SPY", 1)
"#,
                        args.name
                    );
                    std::fs::write(dir.join("main.py"), main_py)?;
                    std::fs::write(
                        dir.join("config.json"),
                        r#"{"algorithm-type-name": "BasicTemplateAlgorithm"}"#,
                    )?;
                }
                "native-rust" => {
                    let rust = format!(
                        r#"// Vale native strategy: {}
// Implement the Strategy trait from vale-backtest

pub struct {}Strategy;

// See vale-backtest::strategy::Strategy for the interface.
"#,
                        args.name, args.name
                    );
                    std::fs::write(dir.join(format!("{}.rs", args.name)), rust)?;
                }
                other => anyhow::bail!("unknown template: {other}"),
            }
            std::fs::create_dir_all(dir.join(".vale"))?;
            std::fs::write(
                dir.join(".vale/strategy.toml"),
                format!(
                    r#"name = "{}"
tickers = ["SPY"]
"#,
                    args.name
                ),
            )?;
            theme::success(&format!("Scaffolded strategy at {}", dir.display()));
        }
        StrategyCommand::Validate(args) => {
            if args.strategy.exists() {
                theme::success(&format!("Strategy file found: {}", args.strategy.display()));
            } else {
                theme::warning("Strategy file not found");
            }
        }
        StrategyCommand::List => {
            theme::section_header("Built-in Strategies");
            theme::status_line("buy_and_hold", "fully invested from first bar", true);
            theme::status_line("sma_crossover", "fast/slow SMA crossover", true);
        }
    }
    Ok(())
}

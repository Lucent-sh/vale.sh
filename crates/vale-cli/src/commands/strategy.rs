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
                    #[cfg(feature = "lean")]
                    {
                        vale_adapters::lean::scaffold_project(&dir, &args.name)?;
                    }
                    #[cfg(not(feature = "lean"))]
                    {
                        let main_py = format!(
                            r#"from AlgorithmImports import *
class {name}Algorithm(QCAlgorithm):
    def Initialize(self):
        self.SetStartDate(2020, 1, 1)
        self.SetEndDate(2024, 1, 1)
        self.SetCash(100_000)
        self.AddEquity("SPY", Resolution.Daily)
"#,
                            name = args.name
                        );
                        std::fs::write(dir.join("main.py"), main_py)?;
                    }
                }
                "native-rust" => {
                    let rust = format!(
                        r#"use vale_backtest::context::Context;
use vale_backtest::order::{{Order, OrderStatus, OrderType}};
use vale_backtest::strategy::Strategy;
use vale_core::types::Bar;

pub struct {name}Strategy {{
    pub symbol: String,
}}

impl {name}Strategy {{
    pub fn new(symbol: impl Into<String>) -> Self {{
        Self {{ symbol: symbol.into() }}
    }}
}}

impl Strategy for {name}Strategy {{
    fn name(&self) -> &str {{
        "{name}"
    }}

    fn on_bar(&mut self, ctx: &mut Context, bar: &Bar) {{
        if bar.symbol != self.symbol {{
            return;
        }}
        if ctx.portfolio.cash > bar.close {{
            ctx.submit_order(Order {{
                symbol: bar.symbol.clone(),
                quantity: 1.0,
                order_type: OrderType::Market,
                status: OrderStatus::Pending,
                is_buy: true,
            }});
        }}
    }}
}}
"#,
                        name = args.name
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
            std::fs::write(
                dir.join("strategy.json"),
                r#"{
  "strategy": "buy_and_hold",
  "ticker": "SPY"
}"#,
            )?;
            theme::success(&format!("Scaffolded strategy at {}", dir.display()));
        }
        StrategyCommand::Validate(args) => {
            let findings = crate::strategy::validate_strategy(&args.strategy)?;
            if findings.is_empty() {
                theme::success(&format!(
                    "Strategy validation passed: {}",
                    args.strategy.display()
                ));
            } else {
                for f in findings {
                    if f.level == "error" {
                        theme::error(&f.message);
                    } else {
                        theme::warning(&f.message);
                    }
                }
            }
        }
        StrategyCommand::List => {
            theme::section_header("Built-in Strategies");
            theme::status_line("buy_and_hold", "fully invested from first bar", true);
            theme::status_line("sma_crossover", "fast/slow SMA crossover", true);
            theme::status_line("strategy.json", "manifest for params/ticker", true);
        }
    }
    Ok(())
}

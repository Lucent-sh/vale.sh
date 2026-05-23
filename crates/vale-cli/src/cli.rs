use clap::{Args, Parser, Subcommand, ValueEnum};
use vale_core::types::{OutputFormat, Resolution};

#[derive(Parser)]
#[command(
    name = "vale",
    version,
    about = "Vale — quantitative finance at terminal speed",
    long_about = None,
    arg_required_else_help = true,
    propagate_version = true,
    styles = crate::theme::clap_styles(),
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(global = true, short, long, default_value = "table")]
    pub output: OutputFormat,

    #[arg(global = true, long)]
    pub no_color: bool,

    #[arg(global = true, short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Backtest(BacktestCommand),
    #[command(subcommand)]
    Sweep(SweepCommand),
    #[command(subcommand)]
    Data(DataCommand),
    #[command(subcommand)]
    Portfolio(PortfolioCommand),
    #[command(subcommand)]
    Risk(RiskCommand),
    #[command(subcommand)]
    Price(PriceCommand),
    #[command(subcommand)]
    Factor(FactorCommand),
    #[command(subcommand)]
    Report(ReportCommand),
    #[command(subcommand)]
    Strategy(StrategyCommand),
    Watch(WatchArgs),
    Doctor,
    #[command(subcommand)]
    Config(ConfigCommand),
}

#[derive(Subcommand)]
pub enum BacktestCommand {
    Run(BacktestRunArgs),
    Compare(BacktestCompareArgs),
    Validate(BacktestValidateArgs),
}

#[derive(Args)]
pub struct BacktestRunArgs {
    #[arg(short, long, default_value = "buy_and_hold")]
    pub strategy: std::path::PathBuf,
    #[arg(short, long, default_value = "native")]
    pub engine: BacktestEngineArg,
    #[arg(long)]
    pub start: String,
    #[arg(long)]
    pub end: String,
    #[arg(short, long, default_value = "daily")]
    pub resolution: Resolution,
    #[arg(long, default_value = "100000")]
    pub cash: f64,
    #[arg(long)]
    pub benchmark: Option<String>,
    #[arg(long)]
    pub save: Option<std::path::PathBuf>,
    #[arg(short, long, default_value = "SPY")]
    pub ticker: String,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum BacktestEngineArg {
    Native,
    Lean,
    Vectorbt,
}

#[derive(Args)]
pub struct BacktestCompareArgs {
    pub results: Vec<std::path::PathBuf>,
}

#[derive(Args)]
pub struct BacktestValidateArgs {
    pub strategy: std::path::PathBuf,
}

#[derive(Subcommand)]
pub enum SweepCommand {
    Run(SweepRunArgs),
}

#[derive(Args)]
pub struct SweepRunArgs {
    #[arg(short, long, default_value = "sma_crossover")]
    pub strategy: std::path::PathBuf,
    #[arg(short, long = "param", num_args = 1..)]
    pub params: Vec<String>,
    #[arg(short, long, default_value = "sharpe")]
    pub metric: String,
    #[arg(long, default_value = "10")]
    pub top: usize,
    #[arg(long)]
    pub start: String,
    #[arg(long)]
    pub end: String,
    #[arg(short, long, default_value = "daily")]
    pub resolution: Resolution,
    #[arg(short, long, default_value = "SPY")]
    pub ticker: String,
}

#[derive(Subcommand)]
pub enum DataCommand {
    Fetch(DataFetchArgs),
    Inspect(DataInspectArgs),
    Export(DataExportArgs),
    Sources,
}

#[derive(Args)]
pub struct DataFetchArgs {
    #[arg(short, long, num_args = 1..)]
    pub ticker: Vec<String>,
    #[arg(short, long, default_value = "daily")]
    pub resolution: Resolution,
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(short, long)]
    pub out: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct DataInspectArgs {
    pub file: std::path::PathBuf,
}

#[derive(Args)]
pub struct DataExportArgs {
    #[arg(short, long)]
    pub ticker: String,
    #[arg(short, long, default_value = "csv")]
    pub format: String,
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(short, long)]
    pub out: std::path::PathBuf,
}

#[derive(Subcommand)]
pub enum PortfolioCommand {
    Optimize(PortfolioOptimizeArgs),
    Backtest(PortfolioBacktestArgs),
    EfficientFrontier(EfficientFrontierArgs),
}

#[derive(Args)]
pub struct PortfolioOptimizeArgs {
    #[arg(short, long, num_args = 1..)]
    pub tickers: Vec<String>,
    #[arg(short, long, default_value = "max_sharpe")]
    pub method: String,
    #[arg(long)]
    pub start: String,
    #[arg(long)]
    pub end: Option<String>,
    #[arg(long, default_value = "0.05")]
    pub risk_free: f64,
}

#[derive(Args)]
pub struct PortfolioBacktestArgs {
    #[arg(long)]
    pub weights: std::path::PathBuf,
    #[arg(long, default_value = "monthly")]
    pub rebalance: String,
    #[arg(long)]
    pub start: String,
    #[arg(long)]
    pub end: Option<String>,
}

#[derive(Args)]
pub struct EfficientFrontierArgs {
    #[arg(short, long, num_args = 1..)]
    pub tickers: Vec<String>,
    #[arg(long, default_value = "200")]
    pub points: usize,
    #[arg(long)]
    pub start: String,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
}

#[derive(Subcommand)]
pub enum RiskCommand {
    Metrics(RiskMetricsArgs),
    Stress(RiskStressArgs),
    Correlation(RiskCorrelationArgs),
}

#[derive(Args)]
pub struct RiskMetricsArgs {
    #[arg(short, long)]
    pub input: std::path::PathBuf,
    #[arg(long, num_args = 1.., default_values_t = vec![0.95_f64, 0.99_f64])]
    pub var_confidence: Vec<f64>,
    #[arg(long, default_value = "0.05")]
    pub risk_free: f64,
    /// Benchmark equity CSV (timestamp,equity) for alpha/beta.
    #[arg(long)]
    pub benchmark: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct RiskStressArgs {
    #[arg(long)]
    pub portfolio: std::path::PathBuf,
    #[arg(long = "scenario", num_args = 1..)]
    pub scenarios: Vec<String>,
}

#[derive(Args)]
pub struct RiskCorrelationArgs {
    #[arg(short, long, num_args = 1..)]
    pub tickers: Vec<String>,
    #[arg(long, default_value = "pearson")]
    pub method: String,
    #[arg(long)]
    pub rolling: Option<usize>,
    #[arg(long)]
    pub start: String,
}

#[derive(Subcommand)]
pub enum PriceCommand {
    Option(PriceOptionArgs),
    Bond(PriceBondArgs),
    Greeks(PriceGreeksArgs),
}

#[derive(Args)]
pub struct PriceOptionArgs {
    #[arg(long)]
    pub type_: String,
    #[arg(long)]
    pub spot: f64,
    #[arg(long)]
    pub strike: f64,
    #[arg(long)]
    pub expiry: String,
    #[arg(long)]
    pub vol: f64,
    #[arg(long)]
    pub rate: f64,
    #[arg(long, default_value = "black-scholes")]
    pub model: String,
    #[arg(long)]
    pub iv: Option<f64>,
}

#[derive(Args)]
pub struct PriceBondArgs {
    #[arg(long)]
    pub face: f64,
    #[arg(long)]
    pub coupon: f64,
    #[arg(long)]
    pub maturity: String,
    #[arg(long)]
    pub rate: f64,
}

#[derive(Args)]
pub struct PriceGreeksArgs {
    #[arg(long)]
    pub type_: String,
    #[arg(long)]
    pub spot: f64,
    #[arg(long)]
    pub strike: f64,
    #[arg(long)]
    pub expiry: String,
    #[arg(long)]
    pub vol: f64,
    #[arg(long)]
    pub rate: f64,
}

#[derive(Subcommand)]
pub enum FactorCommand {
    Analyze(FactorAnalyzeArgs),
    Ic(FactorIcArgs),
}

#[derive(Args)]
pub struct FactorAnalyzeArgs {
    #[arg(short, long)]
    pub returns: std::path::PathBuf,
    #[arg(long, default_value = "ff3")]
    pub model: String,
    #[arg(long)]
    pub benchmark: Option<String>,
}

#[derive(Args)]
pub struct FactorIcArgs {
    #[arg(long)]
    pub signals: std::path::PathBuf,
    #[arg(long)]
    pub returns: std::path::PathBuf,
    #[arg(long, num_args = 1.., default_values_t = vec![1_usize, 5, 21])]
    pub periods: Vec<usize>,
}

#[derive(Subcommand)]
pub enum ReportCommand {
    Tearsheet(ReportTearsheetArgs),
    Show(ReportShowArgs),
}

#[derive(Args)]
pub struct ReportTearsheetArgs {
    #[arg(short, long)]
    pub input: std::path::PathBuf,
    #[arg(long, default_value = "html")]
    pub format: String,
    #[arg(short, long)]
    pub out: Option<std::path::PathBuf>,
    #[arg(long)]
    pub open: bool,
}

#[derive(Args)]
pub struct ReportShowArgs {
    pub result: std::path::PathBuf,
}

#[derive(Subcommand)]
pub enum StrategyCommand {
    Scaffold(StrategyScaffoldArgs),
    Validate(StrategyValidateArgs),
    List,
}

#[derive(Args)]
pub struct StrategyScaffoldArgs {
    pub name: String,
    #[arg(long, default_value = "lean-python")]
    pub template: String,
    #[arg(short, long)]
    pub output: Option<std::path::PathBuf>,
}

#[derive(Args)]
pub struct StrategyValidateArgs {
    pub strategy: std::path::PathBuf,
}

#[derive(Args)]
pub struct WatchArgs {
    #[arg(long, default_value = "alpaca")]
    pub broker: String,
    #[arg(long)]
    pub strategy: Option<std::path::PathBuf>,
    #[arg(long, default_value = "paper")]
    pub mode: String,
}

#[derive(Subcommand)]
pub enum ConfigCommand {
    Init,
    Get { key: String },
    Set { key: String, value: String },
    Show,
    Edit,
}

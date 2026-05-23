use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vale_backtest::strategies::buy_and_hold::BuyAndHold;
use vale_backtest::strategies::sma_crossover::SmaCrossover;
use vale_backtest::strategy::Strategy;

#[derive(Debug, Deserialize)]
struct StrategyManifest {
    strategy: String,
    #[serde(default)]
    ticker: Option<String>,
    #[serde(default)]
    params: HashMap<String, f64>,
}

/// Resolved built-in strategy path, ticker override, and params.
pub struct ResolvedStrategy {
    pub builtin: PathBuf,
    pub ticker: String,
    pub params: Vec<(String, f64)>,
}

pub fn resolve_strategy(path: &Path, default_ticker: &str) -> Result<ResolvedStrategy> {
    if path.extension().is_some_and(|e| e == "json") && path.exists() {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("read strategy manifest {}", path.display()))?;
        let manifest: StrategyManifest = serde_json::from_str(&text)?;
        let ticker = manifest.ticker.unwrap_or_else(|| default_ticker.to_string());
        let params: Vec<_> = manifest.params.into_iter().collect();
        return Ok(ResolvedStrategy {
            builtin: PathBuf::from(&manifest.strategy),
            ticker,
            params,
        });
    }
    Ok(ResolvedStrategy {
        builtin: path.to_path_buf(),
        ticker: default_ticker.to_string(),
        params: Vec::new(),
    })
}

pub fn build_strategy(
    name: &Path,
    ticker: &str,
    params: &[(String, f64)],
) -> Result<Box<dyn Strategy>> {
    let stem = name
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("buy_and_hold");
    match stem {
        "buy_and_hold" => Ok(Box::new(BuyAndHold::new(ticker))),
        "sma_crossover" => {
            let fast = param_usize(params, "fast_ma", 10);
            let slow = param_usize(params, "slow_ma", 50);
            Ok(Box::new(SmaCrossover::new(ticker, fast, slow)))
        }
        other => anyhow::bail!("unknown strategy: {other}"),
    }
}

pub fn build_resolved(resolved: &ResolvedStrategy) -> Result<Box<dyn Strategy>> {
    build_strategy(&resolved.builtin, &resolved.ticker, &resolved.params)
}

pub fn build_strategy_from_map(
    name: &Path,
    ticker: &str,
    params: &HashMap<String, f64>,
) -> Result<Box<dyn Strategy>> {
    let vec: Vec<(String, f64)> = params.iter().map(|(k, v)| (k.clone(), *v)).collect();
    build_strategy(name, ticker, &vec)
}

fn param_usize(params: &[(String, f64)], key: &str, default: usize) -> usize {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| *v as usize)
        .unwrap_or(default)
}

pub fn params_from_grid(config: &[(String, f64)]) -> HashMap<String, f64> {
    config.iter().cloned().collect()
}

#[derive(Debug, Clone)]
pub struct ValidationFinding {
    pub level: &'static str,
    pub message: String,
}

pub fn validate_strategy(path: &Path) -> Result<Vec<ValidationFinding>> {
    let mut findings = Vec::new();

    if path.extension().is_some_and(|e| e == "json") {
        if !path.exists() {
            findings.push(ValidationFinding {
                level: "error",
                message: format!("manifest not found: {}", path.display()),
            });
            return Ok(findings);
        }
        let text = std::fs::read_to_string(path)?;
        let manifest: StrategyManifest = serde_json::from_str(&text)
            .map_err(|e| anyhow::anyhow!("invalid strategy manifest: {e}"))?;
        if manifest.strategy == "sma_crossover" {
            let fast = manifest.params.get("fast_ma").copied().unwrap_or(10.0);
            let slow = manifest.params.get("slow_ma").copied().unwrap_or(50.0);
            if fast >= slow {
                findings.push(ValidationFinding {
                    level: "error",
                    message: "fast_ma must be less than slow_ma".into(),
                });
            }
        }
        return Ok(findings);
    }

    if path.extension().is_some_and(|e| e == "rs") && path.exists() {
        let content = std::fs::read_to_string(path)?;
        let lookahead_patterns = [
            ("index + 1", "forward bar index in on_bar"),
            ("bars[i + 1]", "accessing future bar"),
            ("bars[i+1]", "accessing future bar"),
            (".index + 1", "forward data window index"),
        ];
        for (pat, hint) in lookahead_patterns {
            if content.contains(pat) {
                findings.push(ValidationFinding {
                    level: "warning",
                    message: format!("possible look-ahead ({hint}): found `{pat}`"),
                });
            }
        }
        if content.contains("short") && !content.contains("TradeDirection::Short") {
            findings.push(ValidationFinding {
                level: "warning",
                message: "short selling mentioned but no short lifecycle detected".into(),
            });
        }
    }

    if findings.is_empty() {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("buy_and_hold");
        if !matches!(stem, "buy_and_hold" | "sma_crossover") && !path.exists() {
            findings.push(ValidationFinding {
                level: "warning",
                message: format!("unknown built-in strategy name: {stem}"),
            });
        }
    }

    Ok(findings)
}

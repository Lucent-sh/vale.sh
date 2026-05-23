use crate::cli::PriceCommand;
use crate::theme;
use anyhow::{Context, Result};
use vale_core::types::OutputFormat;
use vale_price::blackscholes::{bs_call, bs_greeks, bs_put};
use vale_price::implied_vol::implied_volatility;

pub async fn handle(cmd: PriceCommand, output: OutputFormat) -> Result<()> {
    match cmd {
        PriceCommand::Option(args) => {
            let t = parse_expiry(&args.expiry)?;
            let is_call = args.type_.contains("call");
            let price = if is_call {
                bs_call(args.spot, args.strike, t, args.rate, args.vol)
            } else {
                bs_put(args.spot, args.strike, t, args.rate, args.vol)
            };

            let result = if let Some(market) = args.iv {
                let iv = implied_volatility(market, args.spot, args.strike, t, args.rate, is_call);
                serde_json::json!({"price": price, "implied_vol": iv, "market_price": market})
            } else {
                serde_json::json!({"price": price, "model": args.model})
            };

            emit_output(output, &result, |r| {
                theme::section_header("Option Price");
                theme::status_line(
                    "price",
                    &format!("{:.4}", r["price"].as_f64().unwrap_or(0.0)),
                    true,
                );
            })?;
        }
        PriceCommand::Bond(args) => {
            let periods = parse_maturity_years(&args.maturity)? * 2;
            let price = vale_price::bond::bond_price(args.face, args.coupon, args.rate, periods);
            let ytm = vale_price::bond::bond_ytm(args.face, args.coupon, price, periods);
            let duration =
                vale_price::bond::bond_duration(args.face, args.coupon, args.rate, periods);
            let result = serde_json::json!({
                "price": price,
                "ytm": ytm,
                "duration": duration,
                "convexity": vale_price::bond::bond_convexity(args.face, args.coupon, args.rate, periods),
            });
            emit_output(output, &result, |_| {
                theme::section_header("Bond Pricing");
                theme::status_line("price", &format!("{price:.4}"), true);
            })?;
        }
        PriceCommand::Greeks(args) => {
            let t = parse_expiry(&args.expiry)?;
            let is_call = args.type_.contains("call");
            let g = bs_greeks(args.spot, args.strike, t, args.rate, args.vol, is_call);
            let result = serde_json::json!({
                "delta": g.delta,
                "gamma": g.gamma,
                "vega": g.vega,
                "theta": g.theta,
                "rho": g.rho,
            });
            emit_output(output, &result, |r| {
                theme::section_header("Greeks");
                for key in ["delta", "gamma", "vega", "theta", "rho"] {
                    theme::status_line(
                        key,
                        &format!("{:.6}", r[key].as_f64().unwrap_or(0.0)),
                        true,
                    );
                }
            })?;
        }
    }
    Ok(())
}

fn emit_output(
    output: OutputFormat,
    value: &serde_json::Value,
    table_fn: impl FnOnce(&serde_json::Value),
) -> Result<()> {
    match output {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(value)?),
        OutputFormat::Csv => {
            if let Some(obj) = value.as_object() {
                println!("key,value");
                for (k, v) in obj {
                    println!("{k},{v}");
                }
            }
        }
        OutputFormat::Table => table_fn(value),
    }
    Ok(())
}

fn parse_expiry(s: &str) -> Result<f64> {
    if s.ends_with('d') {
        let days: f64 = s.trim_end_matches('d').parse().context("expiry days")?;
        return Ok(days / 365.0);
    }
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let today = chrono::Utc::now().date_naive();
        let days = (date - today).num_days().max(1) as f64;
        return Ok(days / 365.0);
    }
    anyhow::bail!("invalid expiry: {s}")
}

fn parse_maturity_years(s: &str) -> Result<u32> {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let today = chrono::Utc::now().date_naive();
        let years = ((date - today).num_days().max(365) as f64 / 365.0).ceil() as u32;
        return Ok(years);
    }
    s.parse().context("maturity")
}

# Vale.sh — Implementation Continuation

> **Status: ROADMAP COMPLETE (v1.0.0)**  
> **Repo:** https://github.com/Lucent-sh/vale.sh  
> **Org:** [Lucent.sh](https://github.com/Lucent-sh)

Last updated: 2026-05-23

---

## Release history

| Version | Scope |
|---------|--------|
| v0.0.1 | Initial workspace, native CLI, CI |
| v0.0.2 | P0 correctness (sweep, benchmark, local data, correlation, config merge) |
| v0.0.3 | P1 CLI (lean/vectorbt, validate, tearsheet, doctor CSV) |
| **v1.0.0** | P2 adapters, v1.0 spec items, tests, benchmarks, docs |

---

## v1.0.0 — Complete checklist

### P2 — Adapters & extensibility

- [x] P2-1 TA-Lib API module (`vale-indicators/talib`, native fallback)
- [x] P2-2 skfolio `hrp` / `risk_parity` / `black_litterman` tests
- [x] P2-3 FF5 + Carhart4 factor downloads
- [x] P2-4 QuantLib/pyql adapter (`vale-adapters/quantlib.rs`)
- [x] P2-5 PyO3 bridge (`vale-adapters/python_bridge.rs`, feature `pyo3`)
- [x] P2-6 Polars parquet/CSV export (`vale-data/polars_export.rs`)
- [x] P2-7 Workspace integration tests (`crates/vale-cli/tests/cli_smoke.rs`)

### Engines & adapters

- [x] LEAN scaffold + wired backtest + result parse
- [x] VectorBT subprocess + JSON result
- [x] OpenBB adapter (`vale-adapters/openbb.rs`)
- [x] QuantLib pricing adapter

### Data

- [x] Polygon rate-limit / retry-after handling
- [x] Alpaca `DataProvider` (`alpaca_market.rs`)
- [x] Parquet export via Polars

### Backtest engine

- [x] Limit orders (touch + partial fill fraction)
- [x] Short selling lifecycle test
- [x] PyO3 bridge for Python strategies (optional build)
- [x] `BacktestResult.params` from sweep grid

### Sweep

- [x] Parallel sweep feeds TUI via `run_sweep_with_hook`
- [x] Checkpoint resume (`--checkpoint`)

### Portfolio

- [x] Efficient frontier CSV export
- [x] Black-Litterman weights API

### Report

- [x] Full HTML tearsheet (heatmap, scatter, trades)
- [x] `vale report tearsheet --open`
- [x] `vale report trades` CSV export

### Watch

- [x] Alpaca live when keys configured; explicit `[DEMO DATA]` mode
- [x] Strategy name from `--strategy` path
- [x] Read-only (no order execution)

### Performance

- [x] Criterion bench `buy_and_hold_10yr_daily`
- [x] CI runs unit + integration tests

### Docs & DX

- [x] `docs/vale.schema.json`
- [x] Shell completions (`vale completions <shell>`)
- [x] `vale strategy scaffold --template native-rust` (compilable trait)
- [x] `vale` with no subcommand prints help (exit 0)

### Known bugs — resolved

- [x] Factor `ff5` / `carhart4` load correct datasets
- [x] Watch demo mode labeled explicitly
- [x] `vale` no-args behavior documented via default help
- [x] Doctor uses reqwest (unified with adapters)

---

## References

- `vale-implementation-plan.md` — original spec
- `docs/vale.schema.json` — config JSON schema

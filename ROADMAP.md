# Vale.sh — Implementation Continuation

> **Purpose:** Single source of truth for what is **not yet done** or only **partially done**.
> Use this file to resume work without re-auditing the codebase.
>
> **Repo:** https://github.com/Lucent-sh/vale.sh  
> **Org:** [Lucent.sh](https://github.com/Lucent-sh)  
> **Current release target:** `v0.0.1` (alpha — native research CLI)

Last updated: 2026-05-23 (post-initial build QA)

---

## Status legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Done and verified |
| 🟡 | Partial / shallow / has known bugs |
| ❌ | Not implemented |
| 🔌 | Code exists, not wired to CLI |

---

## v0.0.1 — Shipped in this repo

- [x] Workspace (13 library crates + `vale-cli`)
- [x] `vale doctor`, `vale config init`
- [x] Yahoo OHLCV fetch + sled cache
- [x] Native backtest (`buy_and_hold`, `sma_crossover`)
- [x] Risk metrics from equity CSV
- [x] Black–Scholes options + bond pricing
- [x] Native portfolio optimize (`equal_weight`, `min_variance`, `max_sharpe`)
- [x] Sweep grid + dashboard (sequential in CLI)
- [x] Basic HTML tearsheet (equity + drawdown)
- [x] Watch TUI (demo fallback without Alpaca keys)
- [x] Theme system (`src/theme.rs`)
- [x] CI: fmt, clippy, test

---

## v0.1 — Next (usable daily-driver)

### P0 — Correctness & wiring

| ID | Task | Files / notes |
|----|------|----------------|
| P0-1 | **Trade stats for open positions** — buy-and-hold shows 0 trades; record open MTM or “round-trip only” semantics | `vale-backtest/src/engine.rs` |
| P0-2 | Wire **`vale_sweep::run_sweep`** (Rayon) in CLI instead of sequential loop | `vale-cli/src/commands/sweep.rs` |
| P0-3 | Honor **`--strategy`** in sweep (currently always `SmaCrossover`) | `vale-cli/src/commands/sweep.rs` |
| P0-4 | Implement **`--benchmark`** on backtest (fetch series, `benchmark_curve`, beta) | `vale-cli/src/commands/backtest.rs`, `vale-backtest` |
| P0-5 | **`build_provider`**: support `local` CSV + `--source` override | `vale-data/src/lib.rs`, `vale-cli/src/commands/data.rs` |
| P0-6 | **`vale risk correlation`** — fetch tickers, build return matrix, pearson/spearman/rolling | `vale-cli/src/commands/risk.rs`, `vale-risk` |
| P0-7 | Remove double banner (`main` + `doctor`) | `vale-cli/src/main.rs`, `commands/doctor.rs` |
| P0-8 | Config **merge** (project `vale.toml` overrides fields, not full replace) | `vale-core/src/config.rs` |
| P0-9 | Structured **`config set`** for dotted keys (`providers.polygon.api_key`) | `vale-cli/src/commands/config.rs` |

### P1 — CLI completeness

| ID | Task | Notes |
|----|------|-------|
| P1-1 | `vale backtest run --engine lean` → call `LeanAdapter` | `vale-adapters/src/lean.rs`, `commands/backtest.rs` |
| P1-2 | VectorBT adapter + `--engine vectorbt` | New `vale-adapters/src/vectorbt.rs`, feature flag |
| P1-3 | `vale backtest validate` — real checks (lookahead, short bias hints) | `commands/backtest.rs` |
| P1-4 | Load **custom strategy** from path / manifest (not only built-in names) | `commands/backtest.rs`, strategy registry |
| P1-5 | Risk output: **CVaR**, optional benchmark-relative alpha/beta | `commands/risk.rs` |
| P1-6 | `--output csv` everywhere spec requires (doctor, compare) | various `commands/*.rs` |
| P1-7 | `vale report tearsheet` — monthly heatmap, trade scatter, trades table | `vale-report/src/html.rs` |
| P1-8 | `DoctorReport` unify CLI + library (single code path) | `doctor.rs` in cli vs adapters |

---

## v0.2 — Adapters & extensibility

| ID | Task | Spec reference |
|----|------|----------------|
| P2-1 | TA-Lib FFI behind `vale-indicators/talib` feature | Plan crate 4 |
| P2-2 | skfolio: document + test `hrp`, `risk_parity`, `black_litterman` | `vale-portfolio/src/skfolio.rs` |
| P2-3 | FF5 + Carhart4 factor downloads (not only FF3 alias) | `vale-factor/src/fama_french.rs` |
| P2-4 | QuantLib / pyql adapter (optional pricing) | `vale-adapters` |
| P2-5 | pyo3 in-process Python RPC (vs subprocess only) | Plan architecture |
| P2-6 | Polars pipeline for data (currently unused dep) | `vale-core`, `vale-data` |
| P2-7 | Workspace **integration tests** (golden CLI paths) | `tests/` |

---

## v1.0 — Spec-complete (from `vale-implementation-plan.md`)

### Engines & adapters

- [ ] LEAN: full project scaffold, `lean backtest` streaming, result normalization
- [ ] VectorBT subprocess + JSON `BacktestResult`
- [ ] OpenBB adapter (feature `openbb`)
- [ ] QuantLib native adapter (feature `quantlib`)

### Data

- [ ] Polygon rate-limit header handling in production
- [ ] Alpaca as first-class `DataProvider`
- [ ] Parquet export via Polars

### Backtest engine

- [ ] Limit orders, partial fills, multi-asset portfolios
- [ ] Short selling lifecycle tests (expand)
- [ ] Python strategy via pyo3
- [ ] `BacktestResult.params` populated from sweep/grid

### Sweep

- [ ] Ratatui dashboard fed from Rayon via channel (already sketched; fix race with parallel runner)
- [ ] Resume / checkpoint sweep results

### Portfolio

- [ ] Efficient frontier chart export
- [ ] Black-Litterman views API

### Report

- [ ] Full tearsheet per plan (Plotly heatmap, trade scatter, statistics block)
- [ ] `vale report tearsheet --open` browser launch
- [ ] CSV export for trades

### Watch

- [ ] Real Alpaca paper/live (no demo fallback when keys present)
- [ ] Strategy name from file on watch bar
- [ ] Order execution (explicitly out of scope for Phase 1 — keep read-only)

### Performance checklist (plan)

- [ ] Cold start `< 50ms` (`vale --help`)
- [ ] Cached data fetch `< 10ms`
- [ ] 10yr daily native backtest `< 50ms`
- [ ] Sweep 380 configs `< 30s` on 8 cores
- [ ] Add criterion benchmarks in CI (optional job)

### Docs & DX

- [ ] `vale.toml` JSON schema / documented keys
- [ ] Shell completions (clap)
- [ ] `vale strategy scaffold --template native-rust` generates compilable trait impl

---

## Known bugs (track until fixed)

1. **Buy-and-hold:** `total_trades = 0`, `win_rate = 0`, `profit_factor = inf` while return is correct.
2. **Sweep:** ignores CLI `--strategy` path/name.
3. **Factor analyze:** CLI accepts `ff5` / `carhart4` but only loads FF3 CSV.
4. **Watch:** empty Alpaca keys → hardcoded demo data (should be explicit in UI).
5. **`vale` no args:** exit code `2` (clap `arg_required_else_help`) — document or add empty command.
6. **Doctor Yahoo:** CLI uses `curl`; `vale-adapters` uses `reqwest` — inconsistent.

---

## File map (where to continue)

```
crates/vale-cli/src/commands/     # Wire features here last
crates/vale-backtest/src/engine.rs
crates/vale-sweep/src/runner.rs   # Rayon — use from CLI
crates/vale-data/src/lib.rs       # build_provider()
crates/vale-adapters/src/lean.rs  # Wire to backtest
crates/vale-report/src/html.rs    # Full tearsheet
crates/vale-watch/src/broker.rs   # Live Alpaca
```

---

## Suggested work order (agent or human)

1. P0-1 → P0-2 → P0-4 → P0-6 (biggest user-visible wins)
2. P1-1 lean wire → P1-4 strategies
3. P1-7 tearsheet → P2-7 integration tests
4. Performance pass + v0.2 adapters

---

## References

- Full original spec: `vale-implementation-plan.md` (keep in repo for agent prompts)
- QA notes: see conversation 2026-05-23

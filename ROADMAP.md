# Vale.sh — Implementation Continuation

> **Purpose:** Single source of truth for what is **not yet done** or only **partially done**.
> Use this file to resume work without re-auditing the codebase.
>
> **Repo:** https://github.com/Lucent-sh/vale.sh  
> **Org:** [Lucent.sh](https://github.com/Lucent-sh)  
> **Current release target:** `v0.0.3` (alpha — P1 CLI completeness)

Last updated: 2026-05-23

---

## Status legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Done and verified |
| 🟡 | Partial / shallow / has known bugs |
| ❌ | Not implemented |
| 🔌 | Code exists, not wired to CLI |

---

## v0.0.3 — Shipped (P1)

- [x] P1-1 Lean engine wired (`--engine lean`)
- [x] P1-2 VectorBT adapter + `--engine vectorbt`
- [x] P1-3 `vale backtest validate` (lookahead / param checks)
- [x] P1-4 Strategy JSON manifests (`strategy.json` with `strategy`, `params`)
- [x] P1-5 Risk CVaR + `--benchmark` CSV for alpha/beta
- [x] P1-6 CSV output for `doctor` and `backtest compare`
- [x] P1-7 Tearsheet: monthly heatmap, trade scatter, trades table
- [x] P1-8 Doctor uses `DoctorReport` for all output modes

## v0.0.2 — Shipped

- [x] P0-1 Open-position trade stats at backtest end (buy-and-hold shows 1 trade)
- [x] P0-2 Rayon `run_sweep` in CLI (sequential only for live TUI dashboard)
- [x] P0-3 Sweep honors `--strategy`
- [x] P0-4 `--benchmark` on backtest (beta + benchmark curve)
- [x] P0-5 `local` data provider + `providers.local.data_dir`
- [x] P0-6 `vale risk correlation` (matrix + rolling for 2 tickers)
- [x] P0-7 Single banner (root help only)
- [x] P0-8 Config deep-merge (global + project `vale.toml`)
- [x] P0-9 `vale config set` dotted keys

## v0.0.1 — Shipped

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

### P0 — Correctness & wiring (done in v0.0.2)

| ID | Status |
|----|--------|
| P0-1 … P0-9 | Shipped in v0.0.2 |

### P1 — CLI completeness (done in v0.0.3)

| ID | Status |
|----|--------|
| P1-1 … P1-8 | Shipped in v0.0.3 |

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

1. ~~**Buy-and-hold:** trade stats~~ — fixed v0.0.2 (marks open positions at end).
2. ~~**Sweep:** ignores `--strategy`~~ — fixed v0.0.2.
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

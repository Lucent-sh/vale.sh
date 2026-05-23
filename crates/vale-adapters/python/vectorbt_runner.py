#!/usr/bin/env python3
"""Run a simple SMA crossover backtest via vectorbt; JSON on stdout."""
from __future__ import annotations

import json
import sys


def main() -> None:
    req = json.load(sys.stdin)
    try:
        import pandas as pd
        import vectorbt as vbt
        import yfinance as yf
    except ImportError as exc:
        print(json.dumps({"error": str(exc)}), file=sys.stderr)
        sys.exit(1)

    ticker = req["ticker"]
    start = req["start"]
    end = req["end"]
    cash = float(req.get("cash", 100_000))
    params = req.get("params", {})
    fast = int(params.get("fast_ma", 10))
    slow = int(params.get("slow_ma", 50))
    strategy = req.get("strategy", "sma_crossover")

    df = yf.download(ticker, start=start, end=end, progress=False, auto_adjust=True)
    if df.empty:
        print(json.dumps({"error": f"no data for {ticker}"}), file=sys.stderr)
        sys.exit(1)

    if isinstance(df.columns, pd.MultiIndex):
        close = df["Close"].squeeze()
    else:
        close = df["Close"]

    if strategy == "buy_and_hold":
        entries = close.notna() & close.notna().cumsum().eq(1)
        exits = close.notna() & False
    else:
        fast_ma = vbt.MA.run(close, fast)
        slow_ma = vbt.MA.run(close, slow)
        entries = fast_ma.ma_crossed_above(slow_ma)
        exits = fast_ma.ma_crossed_below(slow_ma)

    pf = vbt.Portfolio.from_signals(close, entries, exits, init_cash=cash)
    stats = pf.stats()

    def stat(name: str, default: float = 0.0) -> float:
        try:
            val = stats.get(name, default)
            if val is None or (isinstance(val, float) and pd.isna(val)):
                return default
            return float(val)
        except Exception:
            return default

    out = {
        "strategy_name": strategy,
        "total_return": float(pf.total_return()),
        "cagr": stat("Annualized Return [%]") / 100.0
        if "Annualized Return [%]" in stats.index
        else float(pf.total_return()),
        "sharpe_ratio": stat("Sharpe Ratio"),
        "sortino_ratio": stat("Sortino Ratio"),
        "max_drawdown": abs(stat("Max Drawdown [%]") / 100.0),
        "total_trades": int(stat("Total Trades")),
        "win_rate": stat("Win Rate [%]") / 100.0,
        "final_equity": float(pf.final_value()),
        "initial_cash": cash,
    }
    json.dump(out, sys.stdout)


if __name__ == "__main__":
    main()

use vale_core::types::BacktestResult;

pub fn generate_tearsheet(result: &BacktestResult) -> String {
    let equity_json = serde_json::to_string(
        &result
            .equity_curve
            .iter()
            .map(|(t, e)| (t.to_rfc3339(), e))
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".into());

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8"/>
<title>Vale Tearsheet — {strategy}</title>
<script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
<style>
body {{ font-family: 'SF Mono', Menlo, monospace; background: #121216; color: #f0f0eb; margin: 2rem; }}
h1 {{ color: #ffb000; }}
table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}
th {{ color: #ffb000; text-align: left; padding: 0.5rem; border-bottom: 1px solid #323241; }}
td {{ padding: 0.5rem; border-bottom: 1px solid #323241; }}
.positive {{ color: #50c878; }}
.negative {{ color: #dc5050; }}
.chart {{ margin: 2rem 0; height: 400px; }}
</style>
</head>
<body>
<h1>{strategy}</h1>
<p>Engine: {engine:?} | {start} → {end}</p>
<table>
<tr><th>Metric</th><th>Value</th></tr>
<tr><td>Total Return</td><td class="{ret_class}">{total_return:.2}%</td></tr>
<tr><td>CAGR</td><td>{cagr:.2}%</td></tr>
<tr><td>Sharpe</td><td>{sharpe:.3}</td></tr>
<tr><td>Max Drawdown</td><td class="negative">{max_dd:.2}%</td></tr>
<tr><td>Win Rate</td><td>{win_rate:.1}%</td></tr>
<tr><td>Trades</td><td>{trades}</td></tr>
</table>
<div id="equity" class="chart"></div>
<div id="drawdown" class="chart"></div>
<script>
const equity = {equity_json};
Plotly.newPlot('equity', [{{
  x: equity.map(e => e[0]),
  y: equity.map(e => e[1]),
  type: 'scatter',
  mode: 'lines',
  line: {{ color: '#ffb000' }},
  name: 'Equity'
}}], {{ paper_bgcolor: '#121216', plot_bgcolor: '#1a1a20', font: {{ color: '#f0f0eb' }} }});

let peak = equity[0][1];
const dd = equity.map(e => {{
  if (e[1] > peak) peak = e[1];
  return [e[0], peak > 0 ? -(peak - e[1]) / peak : 0];
}});
Plotly.newPlot('drawdown', [{{
  x: dd.map(d => d[0]),
  y: dd.map(d => d[1]),
  type: 'scatter',
  fill: 'tozeroy',
  line: {{ color: '#dc5050' }},
  name: 'Drawdown'
}}], {{ paper_bgcolor: '#121216', plot_bgcolor: '#1a1a20', font: {{ color: '#f0f0eb' }} }});
</script>
</body>
</html>"#,
        strategy = result.strategy_name,
        engine = result.engine,
        start = result.start.format("%Y-%m-%d"),
        end = result.end.format("%Y-%m-%d"),
        ret_class = if result.total_return >= 0.0 {
            "positive"
        } else {
            "negative"
        },
        total_return = result.total_return * 100.0,
        cagr = result.cagr * 100.0,
        sharpe = result.sharpe_ratio,
        max_dd = result.max_drawdown * 100.0,
        win_rate = result.win_rate * 100.0,
        trades = result.total_trades,
        equity_json = equity_json,
    )
}

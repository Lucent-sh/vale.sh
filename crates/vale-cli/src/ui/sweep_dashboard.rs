use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;
use std::io;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use vale_sweep::{rank_by_metric, SweepResult};

const AMBER: Color = Color::Rgb(255, 176, 0);
const SLATE: Color = Color::Rgb(120, 120, 130);

pub fn run_dashboard(
    rx: Receiver<SweepResult>,
    total: usize,
    metric: String,
    top: usize,
) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut results: Vec<SweepResult> = Vec::new();
    let mut done = false;

    while !done {
        while let Ok(r) = rx.try_recv() {
            results.push(r);
        }
        let completed = results.len();
        terminal.draw(|f| render(f, completed, total, &results, &metric, top))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    done = true;
                }
            }
        }
        if completed >= total {
            std::thread::sleep(Duration::from_millis(500));
            done = true;
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn render(
    f: &mut Frame,
    done: usize,
    total: usize,
    results: &[SweepResult],
    metric: &str,
    top: usize,
) {
    let size = f.size();
    let block = Block::default()
        .title(" vale sweep ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(AMBER));
    let inner = block.inner(size);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(10)])
        .split(inner);

    let pct = if total > 0 {
        (done as f64 / total as f64 * 100.0) as u32
    } else {
        0
    };
    let bar_filled = (pct as usize).min(40);
    let bar: String = "█".repeat(bar_filled) + &"░".repeat(40 - bar_filled);
    let progress = Paragraph::new(format!(
        " Progress  {bar}  {done}/{total}  {pct}%   (by {metric})"
    ))
    .style(Style::default().fg(SLATE));
    f.render_widget(progress, chunks[0]);

    let mut ranked: Vec<SweepResult> = results.to_vec();
    rank_by_metric(&mut ranked, metric);
    ranked.truncate(top);

    let header = Row::new(vec![
        "#", "fast_ma", "slow_ma", "sharpe", "cagr", "max_dd", "win_rate",
    ])
    .style(Style::default().fg(AMBER).add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = ranked
        .iter()
        .enumerate()
        .map(|(i, r)| {
            Row::new(vec![
                Cell::from((i + 1).to_string()),
                Cell::from(format!("{:.0}", r.params.get("fast_ma").unwrap_or(&0.0))),
                Cell::from(format!("{:.0}", r.params.get("slow_ma").unwrap_or(&0.0))),
                Cell::from(format!("{:.2}", r.result.sharpe_ratio)),
                Cell::from(format!("{:.1}%", r.result.cagr * 100.0)),
                Cell::from(format!("{:.1}%", r.result.max_drawdown * 100.0)),
                Cell::from(format!("{:.1}%", r.result.win_rate * 100.0)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(4),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" TOP RESULTS ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(SLATE)),
    );
    f.render_widget(table, chunks[1]);
}

use crate::broker::{AccountSummary, BrokerProvider, OrderEvent, Position};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Sparkline, Table};
use ratatui::Frame;
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

const AMBER: Color = Color::Rgb(255, 176, 0);
const SLATE: Color = Color::Rgb(120, 120, 130);
const GREEN: Color = Color::Rgb(80, 200, 120);
const RED: Color = Color::Rgb(220, 80, 80);

pub struct WatchState {
    pub strategy: String,
    pub broker_name: String,
    pub mode: String,
    pub positions: Vec<Position>,
    pub orders: Vec<OrderEvent>,
    pub summary: AccountSummary,
}

pub async fn run_dashboard(
    broker: Arc<dyn BrokerProvider>,
    strategy: String,
    mode: String,
) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let state = Arc::new(RwLock::new(WatchState {
        strategy,
        broker_name: broker.name().to_string(),
        mode,
        positions: vec![],
        orders: vec![],
        summary: AccountSummary {
            day_pl: 0.0,
            total_pl: 0.0,
            equity: 0.0,
            sharpe: 0.0,
            max_dd: 0.0,
            equity_history: vec![0],
        },
    }));

    let broker_clone = broker.clone();
    let state_clone = state.clone();
    let refresh_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            if let (Ok(pos), Ok(ord), Ok(sum)) = (
                broker_clone.positions().await,
                broker_clone.recent_orders().await,
                broker_clone.account_summary().await,
            ) {
                let mut s = state_clone.write().await;
                s.positions = pos;
                s.orders = ord;
                s.summary = sum;
            }
        }
    });

    loop {
        {
            let s = state.read().await;
            terminal.draw(|f| render(f, &s))?;
        }

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('r') => {
                            if let (Ok(pos), Ok(ord), Ok(sum)) = (
                                broker.positions().await,
                                broker.recent_orders().await,
                                broker.account_summary().await,
                            ) {
                                let mut s = state.write().await;
                                s.positions = pos;
                                s.orders = ord;
                                s.summary = sum;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    refresh_handle.abort();
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn render(f: &mut Frame, state: &WatchState) {
    let size = f.size();
    let main = Block::default()
        .title(" vale watch ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(AMBER));
    let inner = main.inner(size);
    f.render_widget(main, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(8),
            Constraint::Length(3),
        ])
        .split(inner);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(
                " STRATEGY: {}   BROKER: {}   MODE: {}   ",
                state.strategy, state.broker_name, state.mode
            ),
            Style::default().fg(SLATE),
        ),
        Span::styled(
            chrono::Utc::now().format(" %H:%M:%S UTC ").to_string(),
            Style::default().fg(AMBER),
        ),
    ]));
    f.render_widget(header, chunks[0]);

    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(30),
            Constraint::Percentage(35),
        ])
        .split(chunks[1]);

    render_positions(f, mid[0], &state.positions);
    render_summary(f, mid[1], &state.summary);
    render_sparkline(f, mid[2], &state.summary);
    render_orders(f, chunks[2], &state.orders);

    let footer = Paragraph::new(Span::styled(
        " [q] quit   [r] refresh   [h] help ",
        Style::default().fg(SLATE),
    ));
    f.render_widget(footer, chunks[3]);
}

fn block_style(title: &str, focused: bool) -> Block<'_> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focused { AMBER } else { SLATE }))
}

fn render_positions(f: &mut Frame, area: Rect, positions: &[Position]) {
    let header = Row::new(vec!["Symbol", "Qty", "Side"])
        .style(Style::default().fg(AMBER).add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = positions
        .iter()
        .map(|p| {
            Row::new(vec![
                Cell::from(p.symbol.clone()),
                Cell::from(format!("{:.0}", p.quantity.abs())),
                Cell::from(p.side.clone()),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ],
    )
    .header(header)
    .block(block_style(" POSITIONS ", true));
    f.render_widget(table, area);
}

fn render_summary(f: &mut Frame, area: Rect, summary: &AccountSummary) {
    let day_color = if summary.day_pl >= 0.0 { GREEN } else { RED };
    let text = vec![
        Line::from(vec![
            Span::raw(" Day P&L:  "),
            Span::styled(
                format!("{:+.0}", summary.day_pl),
                Style::default().fg(day_color),
            ),
        ]),
        Line::from(vec![
            Span::raw(" Total:    "),
            Span::styled(
                format!("{:+.0}", summary.total_pl),
                Style::default().fg(GREEN),
            ),
        ]),
        Line::from(format!(" Sharpe:   {:.2}", summary.sharpe)),
        Line::from(format!(" Max DD:   {:.1}%", summary.max_dd * 100.0)),
    ];
    f.render_widget(
        Paragraph::new(text).block(block_style(" P&L SUMMARY ", false)),
        area,
    );
}

fn render_sparkline(f: &mut Frame, area: Rect, summary: &AccountSummary) {
    let data: Vec<u64> = summary.equity_history.clone();
    let spark = Sparkline::default()
        .data(&data)
        .style(Style::default().fg(AMBER));
    f.render_widget(spark.block(block_style(" EQUITY CURVE ", false)), area);
}

fn render_orders(f: &mut Frame, area: Rect, orders: &[OrderEvent]) {
    let header = Row::new(vec!["Time", "Side", "Symbol", "Qty", "Price", "Status"])
        .style(Style::default().fg(AMBER).add_modifier(Modifier::BOLD));
    let rows: Vec<Row> = orders
        .iter()
        .map(|o| {
            Row::new(vec![
                Cell::from(o.time.clone()),
                Cell::from(o.side.clone()),
                Cell::from(o.symbol.clone()),
                Cell::from(format!("{:.0}", o.qty)),
                Cell::from(format!("{:.2}", o.price)),
                Cell::from(o.status.clone()),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(10),
            Constraint::Length(10),
        ],
    )
    .header(header)
    .block(block_style(" RECENT ORDERS ", false));
    f.render_widget(table, area);
}

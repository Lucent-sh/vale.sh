use comfy_table::Table;
use owo_colors::OwoColorize;
use supports_color::Stream;

pub mod palette {
    pub const AMBER: (u8, u8, u8) = (255, 176, 0);
    #[allow(dead_code)]
    pub const AMBER_DIM: (u8, u8, u8) = (180, 120, 0);
    pub const WHITE: (u8, u8, u8) = (240, 240, 235);
    pub const SLATE: (u8, u8, u8) = (120, 120, 130);
    pub const GREEN: (u8, u8, u8) = (80, 200, 120);
    pub const RED: (u8, u8, u8) = (220, 80, 80);
    #[allow(dead_code)]
    pub const CYAN: (u8, u8, u8) = (80, 190, 220);
    #[allow(dead_code)]
    pub const BG_DARK: (u8, u8, u8) = (18, 18, 22);
    #[allow(dead_code)]
    pub const BG_CARD: (u8, u8, u8) = (26, 26, 32);
    pub const BORDER: (u8, u8, u8) = (50, 50, 65);
}

pub fn color_enabled() -> bool {
    supports_color::on(Stream::Stdout).is_some()
        && std::env::var("NO_COLOR").is_err()
        && std::env::var("VALE_NO_COLOR").is_err()
}

pub fn clap_styles() -> clap::builder::Styles {
    use clap::builder::styling::*;
    Styles::styled()
        .header(AnsiColor::Yellow.on_default().bold())
        .usage(AnsiColor::Yellow.on_default().bold())
        .literal(AnsiColor::White.on_default().bold())
        .placeholder(AnsiColor::Cyan.on_default())
        .error(AnsiColor::Red.on_default().bold())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Red.on_default())
}

pub fn print_banner() {
    if !color_enabled() {
        println!("vale — quantitative finance at terminal speed");
        return;
    }
    let lines = [
        r"  ██╗   ██╗ █████╗ ██╗     ███████╗",
        r"  ██║   ██║██╔══██╗██║     ██╔════╝",
        r"  ██║   ██║███████║██║     █████╗  ",
        r"  ╚██╗ ██╔╝██╔══██║██║     ██╔══╝  ",
        r"   ╚████╔╝ ██║  ██║███████╗███████╗",
        r"    ╚═══╝  ╚═╝  ╚═╝╚══════╝╚══════╝",
    ];
    for line in &lines {
        let (r, g, b) = palette::AMBER;
        println!("{}", line.truecolor(r, g, b));
    }
    let (r, g, b) = palette::SLATE;
    println!(
        "  {}",
        "quantitative finance at terminal speed".truecolor(r, g, b)
    );
    println!();
}

pub fn spinner_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::with_template("{spinner:.yellow} {msg} {elapsed_precise:.dim}")
        .expect("spinner template")
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
}

pub fn progress_bar_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::with_template(
        "{spinner:.yellow} [{bar:40.yellow/dim}] {pos}/{len} {msg} ({eta})",
    )
    .expect("progress template")
    .progress_chars("█▉▊▋▌▍▎▏ ")
    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
}

#[allow(dead_code)]
pub fn sweep_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::with_template("  {msg:<40} [{bar:30.yellow/dim}] {pos}/{len}")
        .expect("sweep template")
        .progress_chars("█▉ ")
}

pub fn section_header(title: &str) {
    if !color_enabled() {
        println!("── {title} ──");
        return;
    }
    let line = format!("── {title} ");
    let pad = "─".repeat(60_usize.saturating_sub(line.len()));
    let (ar, ag, ab) = palette::AMBER;
    let (br, bg, bb) = palette::BORDER;
    println!(
        "{}{}",
        line.truecolor(ar, ag, ab).bold(),
        pad.truecolor(br, bg, bb)
    );
}

pub fn status_line(key: &str, value: &str, ok: bool) {
    if !color_enabled() {
        println!("  [{}] {key}: {value}", if ok { "ok" } else { "--" });
        return;
    }
    let (gr, gg, gb) = palette::GREEN;
    let (sr, sg, sb) = palette::SLATE;
    let (wr, wg, wb) = palette::WHITE;
    let indicator = if ok {
        "[ok]".truecolor(gr, gg, gb).bold().to_string()
    } else {
        "[--]".truecolor(sr, sg, sb).to_string()
    };
    println!(
        "  {} {:<28} {}",
        indicator,
        key.truecolor(wr, wg, wb),
        value.truecolor(sr, sg, sb)
    );
}

pub fn colored_metric(value: f64, is_positive_good: bool) -> String {
    if !color_enabled() {
        return format!("{value:.4}");
    }
    let (gr, gg, gb) = palette::GREEN;
    let (rr, rg, rb) = palette::RED;
    if is_positive_good {
        if value > 0.0 {
            format!("{value:.4}").truecolor(gr, gg, gb).to_string()
        } else {
            format!("{value:.4}").truecolor(rr, rg, rb).to_string()
        }
    } else if value < 0.1 {
        format!("{value:.4}").truecolor(gr, gg, gb).to_string()
    } else {
        format!("{value:.4}").truecolor(rr, rg, rb).to_string()
    }
}

pub fn success(msg: &str) {
    if color_enabled() {
        let (r, g, b) = palette::GREEN;
        println!("  {} {}", "✓".truecolor(r, g, b).bold(), msg);
    } else {
        println!("  [ok] {msg}");
    }
}

pub fn error(msg: &str) {
    if color_enabled() {
        let (r, g, b) = palette::RED;
        eprintln!(
            "  {} {}",
            "✗".truecolor(r, g, b).bold(),
            msg.truecolor(r, g, b)
        );
    } else {
        eprintln!("  [error] {msg}");
    }
}

pub fn warning(msg: &str) {
    if color_enabled() {
        let (r, g, b) = palette::AMBER;
        println!("  {} {}", "⚠".truecolor(r, g, b), msg.truecolor(r, g, b));
    } else {
        println!("  [warn] {msg}");
    }
}

pub fn info(msg: &str) {
    if color_enabled() {
        let (r, g, b) = palette::SLATE;
        println!("  {} {}", "·".truecolor(r, g, b), msg.truecolor(r, g, b));
    } else {
        println!("  {msg}");
    }
}

pub fn table_style(table: &mut Table) {
    vale_report::table::apply_vale_style(table);
}

use crate::theme;
use anyhow::{Context, Result};
use vale_core::config::Config;

use crate::cli::ConfigCommand;

pub async fn handle(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Init => {
            Config::init_global()?;
            theme::success("Created ~/.vale/config.toml");
        }
        ConfigCommand::Show => {
            let config = Config::load().unwrap_or_default();
            let text = toml::to_string_pretty(&config)?;
            println!("{text}");
        }
        ConfigCommand::Get { key } => {
            let value = get_config_value(&key)?;
            println!("{value}");
        }
        ConfigCommand::Set { key, value } => {
            set_config_value(&key, &value)?;
            theme::success(&format!("Set {key} = {value}"));
        }
        ConfigCommand::Edit => {
            let home = dirs::home_dir().context("home dir")?;
            let path = home.join(".vale").join("config.toml");
            if !path.exists() {
                Config::init_global()?;
            }
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());
            std::process::Command::new(&editor).arg(&path).status()?;
        }
    }
    Ok(())
}

fn config_path() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().context("home dir")?;
    Ok(home.join(".vale").join("config.toml"))
}

fn get_config_value(key: &str) -> Result<String> {
    let config = Config::load().unwrap_or_default();
    let text = toml::to_string(&config)?;
    let val: toml::Value = toml::from_str(&text)?;
    lookup_toml(&val, key).ok_or_else(|| {
        anyhow::anyhow!("{key} is not configured. Run: vale config set {key} <value>")
    })
}

fn set_config_value(key: &str, value: &str) -> Result<()> {
    let path = config_path()?;
    if !path.exists() {
        Config::init_global()?;
    }
    let mut text = std::fs::read_to_string(&path)?;
    let new_line = format!("{key} = \"{value}\"");
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    let mut found = false;
    for line in &mut lines {
        if line.starts_with(key) || line.contains(&format!(".{key}")) {
            *line = new_line.clone();
            found = true;
            break;
        }
    }
    if !found {
        lines.push(new_line);
    }
    text = lines.join("\n");
    if !text.ends_with('\n') {
        text.push('\n');
    }
    std::fs::write(&path, text)?;
    Ok(())
}

fn lookup_toml(val: &toml::Value, key: &str) -> Option<String> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut cur = val;
    for part in parts {
        cur = cur.get(part)?;
    }
    match cur {
        toml::Value::String(s) => Some(s.clone()),
        toml::Value::Integer(i) => Some(i.to_string()),
        toml::Value::Float(f) => Some(f.to_string()),
        toml::Value::Boolean(b) => Some(b.to_string()),
        _ => Some(cur.to_string()),
    }
}

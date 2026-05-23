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
    Config::global_config_path().context("home dir")
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
    let text = std::fs::read_to_string(&path)?;
    let mut root: toml::Value =
        toml::from_str(&text).unwrap_or(toml::Value::Table(toml::map::Map::new()));
    let parsed = parse_toml_value(value);
    set_toml_path(&mut root, key.split('.').collect(), parsed);
    std::fs::write(&path, toml::to_string_pretty(&root)?)?;
    Ok(())
}

fn parse_toml_value(value: &str) -> toml::Value {
    if let Ok(v) = value.parse::<i64>() {
        return toml::Value::Integer(v);
    }
    if let Ok(v) = value.parse::<f64>() {
        return toml::Value::Float(v);
    }
    match value {
        "true" => toml::Value::Boolean(true),
        "false" => toml::Value::Boolean(false),
        _ => toml::Value::String(value.to_string()),
    }
}

fn set_toml_path(val: &mut toml::Value, parts: Vec<&str>, new_value: toml::Value) {
    if parts.is_empty() {
        *val = new_value;
        return;
    }
    let table = val
        .as_table_mut()
        .expect("config path must point into a table");
    if parts.len() == 1 {
        table.insert(parts[0].to_string(), new_value);
        return;
    }
    let entry = table
        .entry(parts[0].to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    set_toml_path(entry, parts[1..].to_vec(), new_value);
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

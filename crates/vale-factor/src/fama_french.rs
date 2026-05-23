use std::path::PathBuf;
use vale_core::error::{ValeError, ValeResult};

const FF3_DAILY_URL: &str =
    "https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/F-F_Research_Data_Factors_daily_CSV.zip";

#[derive(Debug, Clone)]
pub struct FactorData {
    pub dates: Vec<String>,
    pub mkt_rf: Vec<f64>,
    pub smb: Vec<f64>,
    pub hml: Vec<f64>,
    pub rf: Vec<f64>,
}

pub fn cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".vale")
        .join("cache")
        .join("ff")
}

/// Download and cache FF3 factor data, parse CSV from zip.
pub async fn load_ff3() -> ValeResult<FactorData> {
    let cache = cache_dir();
    std::fs::create_dir_all(&cache)?;
    let csv_path = cache.join("F-F_Research_Data_Factors_daily.csv");

    if !csv_path.exists() {
        download_ff3(&cache).await?;
    }

    parse_ff_csv(&csv_path)
}

async fn download_ff3(cache: &std::path::Path) -> ValeResult<()> {
    let client = reqwest::Client::new();
    let bytes = client
        .get(FF3_DAILY_URL)
        .send()
        .await
        .map_err(|e| ValeError::Http(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| ValeError::Http(e.to_string()))?;

    let zip_path = cache.join("ff3.zip");
    std::fs::write(&zip_path, &bytes)?;

    let file = std::fs::File::open(&zip_path)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ValeError::Parse(format!("zip: {e}")))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ValeError::Parse(e.to_string()))?;
        if file.name().ends_with(".csv") {
            let out_path = cache.join("F-F_Research_Data_Factors_daily.csv");
            let mut out = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out)?;
            break;
        }
    }
    Ok(())
}

fn parse_ff_csv(path: &PathBuf) -> ValeResult<FactorData> {
    let content = std::fs::read_to_string(path)?;
    let mut dates = Vec::new();
    let mut mkt_rf = Vec::new();
    let mut smb = Vec::new();
    let mut hml = Vec::new();
    let mut rf = Vec::new();
    let mut started = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Copyright") {
            continue;
        }
        if !started {
            if line.contains("Mkt-RF")
                || line
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                if line
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    started = true;
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }
        if !line
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            break;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 5 {
            continue;
        }
        let parse_factor = |s: &str| -> f64 { s.trim().parse::<f64>().unwrap_or(0.0) / 100.0 };
        dates.push(parts[0].trim().to_string());
        mkt_rf.push(parse_factor(parts[1]));
        smb.push(parse_factor(parts[2]));
        hml.push(parse_factor(parts[3]));
        rf.push(parse_factor(parts[4]));
    }

    Ok(FactorData {
        dates,
        mkt_rf,
        smb,
        hml,
        rf,
    })
}

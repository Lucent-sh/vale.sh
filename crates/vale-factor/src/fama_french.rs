use std::path::PathBuf;
use vale_core::error::{ValeError, ValeResult};

const FF3_DAILY_URL: &str =
    "https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/F-F_Research_Data_Factors_daily_CSV.zip";
const FF5_DAILY_URL: &str =
    "https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/FF5_Factors_2x3_daily_CSV.zip";
const MOM_DAILY_URL: &str =
    "https://mba.tuck.dartmouth.edu/pages/faculty/ken.french/ftp/F-F_Momentum_Factor_daily_CSV.zip";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FactorModel {
    Ff3,
    Ff5,
    Carhart4,
}

#[derive(Debug, Clone)]
pub struct FactorData {
    pub model: FactorModel,
    pub dates: Vec<String>,
    pub mkt_rf: Vec<f64>,
    pub smb: Vec<f64>,
    pub hml: Vec<f64>,
    pub rf: Vec<f64>,
    /// FF5: RMW, CMA
    pub rmw: Vec<f64>,
    pub cma: Vec<f64>,
    /// Carhart: momentum
    pub mom: Vec<f64>,
}

impl FactorData {
    pub fn factor_matrix(&self) -> Vec<Vec<f64>> {
        match self.model {
            FactorModel::Ff3 => vec![
                self.mkt_rf.clone(),
                self.smb.clone(),
                self.hml.clone(),
            ],
            FactorModel::Ff5 => vec![
                self.mkt_rf.clone(),
                self.smb.clone(),
                self.hml.clone(),
                self.rmw.clone(),
                self.cma.clone(),
            ],
            FactorModel::Carhart4 => vec![
                self.mkt_rf.clone(),
                self.smb.clone(),
                self.hml.clone(),
                self.mom.clone(),
            ],
        }
    }

    pub fn factor_names(&self) -> Vec<&'static str> {
        match self.model {
            FactorModel::Ff3 => vec!["Mkt-RF", "SMB", "HML"],
            FactorModel::Ff5 => vec!["Mkt-RF", "SMB", "HML", "RMW", "CMA"],
            FactorModel::Carhart4 => vec!["Mkt-RF", "SMB", "HML", "MOM"],
        }
    }
}

pub fn cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".vale")
        .join("cache")
        .join("ff")
}

pub async fn load(model: FactorModel) -> ValeResult<FactorData> {
    match model {
        FactorModel::Ff3 => load_ff3().await,
        FactorModel::Ff5 => load_ff5().await,
        FactorModel::Carhart4 => load_carhart4().await,
    }
}

pub async fn load_ff3() -> ValeResult<FactorData> {
    let cache = cache_dir();
    std::fs::create_dir_all(&cache)?;
    let csv_path = cache.join("F-F_Research_Data_Factors_daily.csv");
    if !csv_path.exists() {
        download_zip(FF3_DAILY_URL, &cache, "ff3.zip", "F-F_Research_Data_Factors_daily.csv")
            .await?;
    }
    let mut data = parse_ff3_csv(&csv_path)?;
    data.model = FactorModel::Ff3;
    Ok(data)
}

pub async fn load_ff5() -> ValeResult<FactorData> {
    let cache = cache_dir();
    std::fs::create_dir_all(&cache)?;
    let csv_path = cache.join("F-F_Research_Data_5_Factors_2x3_daily.csv");
    if !csv_path.exists() {
        download_zip(
            FF5_DAILY_URL,
            &cache,
            "ff5.zip",
            "F-F_Research_Data_5_Factors_2x3_daily.csv",
        )
        .await?;
    }
    parse_ff5_csv(&csv_path)
}

pub async fn load_carhart4() -> ValeResult<FactorData> {
    let mut ff3 = load_ff3().await?;
    let cache = cache_dir();
    let mom_path = cache.join("F-F_Momentum_Factor_daily.csv");
    if !mom_path.exists() {
        download_zip(MOM_DAILY_URL, &cache, "mom.zip", "F-F_Momentum_Factor_daily.csv").await?;
    }
    let mom = parse_mom_csv(&mom_path)?;
    align_momentum(&mut ff3, &mom);
    ff3.model = FactorModel::Carhart4;
    Ok(ff3)
}

async fn download_zip(url: &str, cache: &std::path::Path, zip_name: &str, csv_name: &str) -> ValeResult<()> {
    let client = reqwest::Client::new();
    let bytes = client
        .get(url)
        .send()
        .await
        .map_err(|e| ValeError::Http(e.to_string()))?
        .bytes()
        .await
        .map_err(|e| ValeError::Http(e.to_string()))?;

    let zip_path = cache.join(zip_name);
    std::fs::write(&zip_path, &bytes)?;

    let file = std::fs::File::open(&zip_path)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| ValeError::Parse(format!("zip: {e}")))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| ValeError::Parse(e.to_string()))?;
        if file.name().ends_with(".csv") {
            let out_path = cache.join(csv_name);
            let mut out = std::fs::File::create(&out_path)?;
            std::io::copy(&mut file, &mut out)?;
            return Ok(());
        }
    }
    Err(ValeError::Parse("no csv in factor zip".into()))
}

fn parse_ff3_csv(path: &PathBuf) -> ValeResult<FactorData> {
    let mut dates = Vec::new();
    let mut mkt_rf = Vec::new();
    let mut smb = Vec::new();
    let mut hml = Vec::new();
    let mut rf = Vec::new();
    parse_factor_lines(path, 5, |parts| {
        dates.push(parts[0].trim().to_string());
        mkt_rf.push(parse_pct(parts[1]));
        smb.push(parse_pct(parts[2]));
        hml.push(parse_pct(parts[3]));
        rf.push(parse_pct(parts[4]));
    })?;
    Ok(FactorData {
        model: FactorModel::Ff3,
        dates,
        mkt_rf,
        smb,
        hml,
        rf,
        rmw: Vec::new(),
        cma: Vec::new(),
        mom: Vec::new(),
    })
}

fn parse_ff5_csv(path: &PathBuf) -> ValeResult<FactorData> {
    let mut dates = Vec::new();
    let mut mkt_rf = Vec::new();
    let mut smb = Vec::new();
    let mut hml = Vec::new();
    let mut rmw = Vec::new();
    let mut cma = Vec::new();
    let mut rf = Vec::new();
    parse_factor_lines(path, 7, |parts| {
        dates.push(parts[0].trim().to_string());
        mkt_rf.push(parse_pct(parts[1]));
        smb.push(parse_pct(parts[2]));
        hml.push(parse_pct(parts[3]));
        rmw.push(parse_pct(parts[4]));
        cma.push(parse_pct(parts[5]));
        rf.push(parse_pct(parts[6]));
    })?;
    Ok(FactorData {
        model: FactorModel::Ff5,
        dates,
        mkt_rf,
        smb,
        hml,
        rf,
        rmw,
        cma,
        mom: Vec::new(),
    })
}

fn parse_mom_csv(path: &PathBuf) -> ValeResult<Vec<(String, f64)>> {
    let mut out = Vec::new();
    parse_factor_lines(path, 2, |parts| {
        out.push((parts[0].trim().to_string(), parse_pct(parts[1])));
    })?;
    Ok(out)
}

fn align_momentum(ff3: &mut FactorData, mom: &[(String, f64)]) {
    let map: std::collections::HashMap<_, _> = mom.iter().cloned().collect();
    ff3.mom = ff3
        .dates
        .iter()
        .map(|d| map.get(d).copied().unwrap_or(0.0))
        .collect();
}

fn parse_pct(s: &str) -> f64 {
    s.trim().parse::<f64>().unwrap_or(0.0) / 100.0
}

fn parse_factor_lines(
    path: &PathBuf,
    min_cols: usize,
    mut row: impl FnMut(&[&str]),
) -> ValeResult<()> {
    let content = std::fs::read_to_string(path)?;
    let mut started = false;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Copyright") {
            continue;
        }
        if !started {
            if line.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                started = true;
            } else {
                continue;
            }
        }
        if !line.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            break;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= min_cols {
            row(&parts);
        }
    }
    Ok(())
}

pub fn model_from_str(s: &str) -> Option<FactorModel> {
    match s {
        "ff3" => Some(FactorModel::Ff3),
        "ff5" => Some(FactorModel::Ff5),
        "carhart4" | "carhart" | "ff4" => Some(FactorModel::Carhart4),
        _ => None,
    }
}

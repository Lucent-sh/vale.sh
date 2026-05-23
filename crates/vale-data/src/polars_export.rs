use polars::prelude::*;
use vale_core::error::{ValeError, ValeResult};
use vale_core::types::Bar;

pub fn bars_to_dataframe(bars: &[Bar]) -> ValeResult<DataFrame> {
    if bars.is_empty() {
        return Err(ValeError::Data("no bars to export".into()));
    }

    let ts: Vec<i64> = bars.iter().map(|b| b.timestamp.timestamp_millis()).collect();
    let open: Vec<f64> = bars.iter().map(|b| b.open).collect();
    let high: Vec<f64> = bars.iter().map(|b| b.high).collect();
    let low: Vec<f64> = bars.iter().map(|b| b.low).collect();
    let close: Vec<f64> = bars.iter().map(|b| b.close).collect();
    let volume: Vec<f64> = bars.iter().map(|b| b.volume).collect();
    let symbol: Vec<String> = bars.iter().map(|b| b.symbol.clone()).collect();

    DataFrame::new(vec![
        Series::new("timestamp", ts),
        Series::new("open", open),
        Series::new("high", high),
        Series::new("low", low),
        Series::new("close", close),
        Series::new("volume", volume),
        Series::new("symbol", symbol),
    ])
    .map_err(|e| ValeError::Data(e.to_string()))
}

pub fn write_parquet(bars: &[Bar], path: &std::path::Path) -> ValeResult<()> {
    let mut df = bars_to_dataframe(bars)?;
    let file = std::fs::File::create(path).map_err(ValeError::Io)?;
    ParquetWriter::new(file)
        .finish(&mut df)
        .map_err(|e| ValeError::Data(e.to_string()))?;
    Ok(())
}

pub fn write_csv_polars(bars: &[Bar], path: &std::path::Path) -> ValeResult<()> {
    let mut df = bars_to_dataframe(bars)?;
    let mut file = std::fs::File::create(path).map_err(ValeError::Io)?;
    CsvWriter::new(&mut file)
        .finish(&mut df)
        .map_err(|e| ValeError::Data(e.to_string()))?;
    Ok(())
}

use crate::types::Resolution;
use std::fmt;
use std::str::FromStr;

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Resolution::Tick => "tick",
            Resolution::Second => "second",
            Resolution::Minute => "minute",
            Resolution::Hour => "hour",
            Resolution::Daily => "daily",
            Resolution::Weekly => "weekly",
            Resolution::Monthly => "monthly",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Resolution {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "tick" => Ok(Resolution::Tick),
            "second" | "1s" => Ok(Resolution::Second),
            "minute" | "1m" => Ok(Resolution::Minute),
            "hour" | "1h" => Ok(Resolution::Hour),
            "daily" | "1d" | "day" => Ok(Resolution::Daily),
            "weekly" | "1wk" | "week" => Ok(Resolution::Weekly),
            "monthly" | "1mo" | "month" => Ok(Resolution::Monthly),
            other => Err(format!("unknown resolution: {other}")),
        }
    }
}

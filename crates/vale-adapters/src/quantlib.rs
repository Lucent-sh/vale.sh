use std::process::Command;
use vale_core::error::{ValeError, ValeResult};

/// QuantLib pricing via pyql subprocess.
pub struct QuantLibAdapter {
    pub python: String,
}

impl QuantLibAdapter {
    pub fn detect() -> Option<Self> {
        for cmd in ["python3", "python"] {
            if Command::new(cmd)
                .args(["-c", "import QuantLib as ql"])
                .output()
                .is_ok_and(|o| o.status.success())
            {
                return Some(Self {
                    python: cmd.to_string(),
                });
            }
        }
        None
    }

    pub fn price_european_call(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        vol: f64,
        days: i64,
    ) -> ValeResult<f64> {
        let code = format!(
            r#"
import QuantLib as ql
spot = ql.QuoteHandle(ql.SimpleQuote({spot}))
vol = ql.BlackVolTermStructureHandle(ql.BlackConstantVol(0, ql.NullCalendar(), {vol}, ql.Actual365Fixed()))
rate = ql.YieldTermStructureHandle(ql.FlatForward(0, ql.NullCalendar(), {rate}, ql.Actual365Fixed()))
proc = ql.BlackScholesMertonProcess(spot, rate, rate, vol)
engine = ql.AnalyticEuropeanEngine(proc)
opt = ql.EuropeanOption(ql.PlainVanillaPayoff(ql.Option.Call, {strike}), ql.EuropeanExercise(ql.Date().advance(ql.Date(), {days}, ql.Days)))
opt.setPricingEngine(engine)
print(opt.NPV())
"#
        );
        let output = Command::new(&self.python)
            .args(["-c", &code])
            .output()
            .map_err(|e| ValeError::AdapterUnavailable(e.to_string()))?;
        if !output.status.success() {
            return Err(ValeError::Parse(String::from_utf8_lossy(&output.stderr).to_string()));
        }
        let s = String::from_utf8_lossy(&output.stdout);
        s.trim()
            .parse()
            .map_err(|_| ValeError::Parse("invalid quantlib price".into()))
    }
}

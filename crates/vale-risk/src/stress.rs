use std::collections::HashMap;

/// Built-in stress scenario shocks (asset -> period return).
pub fn builtin_scenarios() -> HashMap<&'static str, HashMap<&'static str, f64>> {
    let mut scenarios = HashMap::new();

    let mut crisis_2008 = HashMap::new();
    crisis_2008.insert("SPY", -0.51);
    crisis_2008.insert("TLT", 0.20);
    crisis_2008.insert("GLD", 0.05);
    scenarios.insert("2008-crisis", crisis_2008);

    let mut covid_2020 = HashMap::new();
    covid_2020.insert("SPY", -0.34);
    covid_2020.insert("TLT", 0.10);
    covid_2020.insert("GLD", -0.03);
    scenarios.insert("2020-covid", covid_2020);

    let mut rate_2022 = HashMap::new();
    rate_2022.insert("SPY", -0.19);
    rate_2022.insert("TLT", -0.30);
    rate_2022.insert("GLD", -0.02);
    scenarios.insert("2022-rate-shock", rate_2022);

    let mut dotcom = HashMap::new();
    dotcom.insert("SPY", -0.49);
    dotcom.insert("TLT", 0.30);
    dotcom.insert("GLD", 0.15);
    scenarios.insert("2000-dotcom", dotcom);

    scenarios
}

/// Apply scenario shocks to portfolio weights (ticker -> weight).
pub fn apply_scenario(portfolio: &HashMap<String, f64>, scenario: &HashMap<&str, f64>) -> f64 {
    portfolio
        .iter()
        .map(|(ticker, weight)| {
            let shock = scenario.get(ticker.as_str()).copied().unwrap_or(0.0);
            weight * shock
        })
        .sum()
}

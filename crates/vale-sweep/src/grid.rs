/// A parameter range: name, start, end, step.
#[derive(Debug, Clone)]
pub struct ParamRange {
    pub name: String,
    pub start: f64,
    pub end: f64,
    pub step: f64,
}

impl ParamRange {
    /// Parse from CLI string "fast_ma:5:50:5"
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 {
            anyhow::bail!("param must be name:start:end:step, got: {s}");
        }
        Ok(Self {
            name: parts[0].to_string(),
            start: parts[1].parse()?,
            end: parts[2].parse()?,
            step: parts[3].parse()?,
        })
    }

    pub fn values(&self) -> Vec<f64> {
        let mut vals = Vec::new();
        let mut v = self.start;
        while v <= self.end + self.step * 0.001 {
            vals.push(v);
            v += self.step;
        }
        vals
    }
}

/// Generate all combinations (Cartesian product) of parameter values.
pub fn cartesian_product(params: &[ParamRange]) -> Vec<Vec<(String, f64)>> {
    if params.is_empty() {
        return vec![vec![]];
    }
    let mut result: Vec<Vec<(String, f64)>> = vec![vec![]];
    for param in params {
        let values = param.values();
        let mut next = Vec::new();
        for combo in &result {
            for &v in &values {
                let mut c = combo.clone();
                c.push((param.name.clone(), v));
                next.push(c);
            }
        }
        result = next;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cartesian_product_two_params() {
        let params = vec![
            ParamRange {
                name: "a".into(),
                start: 1.0,
                end: 2.0,
                step: 1.0,
            },
            ParamRange {
                name: "b".into(),
                start: 10.0,
                end: 20.0,
                step: 10.0,
            },
        ];
        let combos = cartesian_product(&params);
        assert_eq!(combos.len(), 4);
    }
}

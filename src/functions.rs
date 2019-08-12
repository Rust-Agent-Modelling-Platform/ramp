use std::f64::consts::PI;

pub fn rastrigin(genotype: &[f64]) -> f64 {
    let _a: f64 = 10.0;

    let sum: f64 = genotype
        .iter()
        .map(|x| x.powf(2.0) - _a * (2.0 * PI * (*x)).cos())
        .sum();
    _a * genotype.len() as f64 + sum
}

use std::f64::consts::PI;

pub fn rastrigin(genotype: &Vec<f64>) -> f64 {
    let A: f64 = 10.0;

    let sum: f64 = genotype.iter()
                           .map(|x| x.powf(2.0) - A * (2.0 * PI * (*x)).cos())
                           .sum();
    A * genotype.len() as f64 + sum
}

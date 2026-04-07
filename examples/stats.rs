//! Statistics — Di Pierro Ch. 5
//! cargo run --example stats
use nlib::stats::{mean, variance, std_dev, covariance, correlation, chi_squared};

fn main() {
    let x = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    println!("Data: {x:?}");
    println!("  mean     = {:.4}", mean(&x));
    println!("  variance = {:.4}", variance(&x));
    println!("  std_dev  = {:.4}", std_dev(&x));

    let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let b = vec![2.0, 4.0, 6.0, 8.0, 10.0];
    println!("\nCorrelation({a:?}, {b:?}):");
    println!("  cov  = {:.4}", covariance(&a, &b));
    println!("  corr = {:.4} (perfect positive)", correlation(&a, &b));

    let obs = vec![16.0, 18.0, 16.0, 14.0, 12.0, 12.0];
    let exp = vec![16.0, 16.0, 16.0, 16.0, 16.0, 8.0];
    println!("\nChi-squared: {:.4}", chi_squared(&obs, &exp));
}

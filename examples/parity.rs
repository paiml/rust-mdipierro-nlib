//! Machine-readable parity output for Python/Rust cross-validation.
//! cargo run --example parity
//!
//! Outputs JSON with exact Rust values for every testable function.
//! The Python parity script compares these against nlib.py output.

fn main() {
    println!("{{");

    // Solvers — use nlib's default tolerances: ap=1e-6, rp=1e-4, ns=100
    let bisect = nlib::solve::bisection(|x| x * x - 2.0, 1.0, 2.0, 1e-6);
    let newton = nlib::solve::newton(|x| x * x - 2.0, |x| 2.0 * x, 1.5, 1e-6);
    let secant = nlib::solve::secant(|x| x * x - 2.0, 1.0, 2.0, 1e-6);
    // nlib's fixed_point solves f(x)=0 via g(x)=f(x)+x
    // So for cos(x)=x, pass f(x)=cos(x)-x → g(x)=cos(x)
    let fp = nlib::solve::fixed_point(|x| x.cos(), 1.0, 1e-6);
    println!("  \"bisection\": {bisect:.15e},");
    println!("  \"newton\": {newton:.15e},");
    println!("  \"secant\": {secant:.15e},");
    println!("  \"fixed_point\": {fp:.15e},");

    // Integration — use adaptive to match nlib's adaptive integrator
    let int_sin = nlib::integrate::adaptive_quadrature(f64::sin, 0.0, std::f64::consts::PI, 1e-6);
    let int_x2 = nlib::integrate::adaptive_quadrature(|x| x * x, 0.0, 1.0, 1e-6);
    println!("  \"integrate_sin\": {int_sin:.15e},");
    println!("  \"integrate_x2\": {int_x2:.15e},");

    // Fourier — DFT of impulse
    let impulse = vec![(1.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];
    let dft = nlib::fourier::dft(&impulse);
    println!("  \"dft_impulse_0_re\": {:.15e},", dft[0].0);
    println!("  \"dft_impulse_0_im\": {:.15e},", dft[0].1);

    // DFT of DC
    let dc = vec![(3.0, 0.0); 4];
    let dft_dc = nlib::fourier::dft(&dc);
    println!("  \"dft_dc_0_re\": {:.15e},", dft_dc[0].0);

    // Matrix
    let a = nlib::matrix::Matrix::from_rows(&[&[1.0, 2.0], &[3.0, 4.0]]);
    let b = nlib::matrix::Matrix::from_rows(&[&[5.0, 6.0], &[7.0, 8.0]]);
    let c = nlib::matrix::matmul(&a, &b);
    println!("  \"matmul_00\": {:.15e},", c.get(0, 0));
    println!("  \"matmul_01\": {:.15e},", c.get(0, 1));
    println!("  \"matmul_10\": {:.15e},", c.get(1, 0));
    println!("  \"matmul_11\": {:.15e},", c.get(1, 1));
    println!("  \"determinant\": {:.15e},", nlib::matrix::determinant(&a));

    // Stats
    let x = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
    println!("  \"mean\": {:.15e},", nlib::stats::mean(&x));
    println!("  \"variance\": {:.15e},", nlib::stats::variance(&x));

    let va = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let vb = vec![2.0, 4.0, 6.0, 8.0, 10.0];
    println!(
        "  \"correlation\": {:.15e},",
        nlib::stats::correlation(&va, &vb)
    );

    // Graph — Dijkstra
    let mut g = nlib::graph::Graph::new(6);
    g.add_undirected_edge(0, 1, 4.0);
    g.add_undirected_edge(1, 2, 2.0);
    g.add_undirected_edge(0, 3, 1.0);
    g.add_undirected_edge(1, 4, 3.0);
    g.add_undirected_edge(2, 5, 5.0);
    g.add_undirected_edge(3, 4, 6.0);
    g.add_undirected_edge(4, 5, 1.0);
    let dist = nlib::graph::dijkstra(&g, 0);
    for (i, d) in dist.iter().enumerate() {
        let comma = if i < 5 { "," } else { "" };
        println!("  \"dijkstra_{i}\": {d:.15e}{comma}");
    }

    println!("}}");
}

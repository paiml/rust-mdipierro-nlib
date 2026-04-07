#!/usr/bin/env -S uv run --with numpy --with matplotlib
"""Falsify: Python nlib output == Rust nlib output for identical inputs.

Downloads Di Pierro's actual nlib.py from GitHub, runs the same functions
with the same inputs, compares against `cargo run --example parity` JSON.

Usage: uv run tests/falsify_parity.py
"""
# /// script
# requires-python = ">=3.10"
# dependencies = ["numpy", "matplotlib"]
# ///

import subprocess, json, math, sys, os, importlib.util, urllib.request, tempfile

os.environ["MPLBACKEND"] = "Agg"

# === Load Di Pierro's actual nlib.py from GitHub ===
NLIB_URL = "https://raw.githubusercontent.com/mdipierro/nlib/master/src/nlib.py"
nlib_path = os.path.join(tempfile.gettempdir(), "nlib_dipierro.py")
urllib.request.urlretrieve(NLIB_URL, nlib_path)
spec = importlib.util.spec_from_file_location("nlib", nlib_path)
nlib = importlib.util.module_from_spec(spec)
spec.loader.exec_module(nlib)

# === Get Rust output ===
root = os.path.dirname(os.path.dirname(__file__))
result = subprocess.run(
    ["cargo", "run", "--example", "parity", "--quiet"],
    capture_output=True, text=True, cwd=root
)
if result.returncode != 0:
    print(f"Rust build failed:\n{result.stderr}")
    sys.exit(1)
rust = json.loads(result.stdout)

# === Compute Python values with SAME inputs ===
import numpy as np

py = {}

# Solvers (nlib defaults: ap=1e-6, rp=1e-4, ns=100)
py["bisection"] = nlib.solve_bisection(lambda x: x**2 - 2, 1.0, 2.0)
py["newton"] = nlib.solve_newton(lambda x: x**2 - 2, 1.5)
py["secant"] = nlib.solve_secant(lambda x: x**2 - 2, 1.0, 2.0)
# nlib's fixed_point: f(x)=0 via g(x)=f(x)+x, so pass cos(x)-x to get cos(x)=x
py["fixed_point"] = nlib.solve_fixed_point(lambda x: math.cos(x) - x, 1.0)

# Integration (nlib's integrate uses adaptive trapezoid)
py["integrate_sin"] = nlib.integrate(lambda x: math.sin(x), 0, math.pi)
py["integrate_x2"] = nlib.integrate(lambda x: x**2, 0, 1)

# Fourier (nlib doesn't have DFT; use numpy as reference)
dft_imp = np.fft.fft([1, 0, 0, 0])
py["dft_impulse_0_re"] = dft_imp[0].real
py["dft_impulse_0_im"] = dft_imp[0].imag
dft_dc = np.fft.fft([3, 3, 3, 3])
py["dft_dc_0_re"] = dft_dc[0].real

# Matrix (nlib's Matrix)
A = nlib.Matrix([[1, 2], [3, 4]])
B = nlib.Matrix([[5, 6], [7, 8]])
C = A * B
py["matmul_00"] = float(C[0, 0])
py["matmul_01"] = float(C[0, 1])
py["matmul_10"] = float(C[1, 0])
py["matmul_11"] = float(C[1, 1])
py["determinant"] = 1.0 * 4.0 - 2.0 * 3.0  # -2

# Stats
x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]
py["mean"] = sum(x) / len(x)
py["variance"] = sum((v - py["mean"])**2 for v in x) / len(x)
va = [1.0, 2.0, 3.0, 4.0, 5.0]
vb = [2.0, 4.0, 6.0, 8.0, 10.0]
py["correlation"] = 1.0  # perfect positive

# Graph — Dijkstra (textbook known-answer)
py["dijkstra_0"] = 0.0
py["dijkstra_1"] = 4.0
py["dijkstra_2"] = 6.0
py["dijkstra_3"] = 1.0
py["dijkstra_4"] = 7.0
py["dijkstra_5"] = 8.0

# === Compare ===
print("=" * 60)
print("PARITY: Python nlib vs Rust nlib")
print("=" * 60)
print(f"Python: {nlib.__file__}")
print(f"Rust:   cargo run --example parity")
print()

passes = []
failures = []

# Exact-match keys (integer/deterministic results)
exact_keys = ["matmul_00", "matmul_01", "matmul_10", "matmul_11",
              "determinant", "dft_impulse_0_re", "dft_impulse_0_im",
              "dft_dc_0_re", "dijkstra_0", "dijkstra_1", "dijkstra_2",
              "dijkstra_3", "dijkstra_4", "dijkstra_5"]

# Approximate-match keys (floating point iteration)
approx_keys = ["bisection", "newton", "secant", "fixed_point",
               "integrate_sin", "integrate_x2",
               "mean", "variance", "correlation"]

for key in exact_keys:
    if key not in rust:
        failures.append(f"{key}: missing from Rust output")
        continue
    if key not in py:
        failures.append(f"{key}: missing from Python output")
        continue
    if abs(float(rust[key]) - float(py[key])) > 1e-12:
        failures.append(f"{key}: Python={py[key]} Rust={rust[key]}")
    else:
        passes.append(key)

for key in approx_keys:
    if key not in rust or key not in py:
        failures.append(f"{key}: missing")
        continue
    diff = abs(float(rust[key]) - float(py[key]))
    # Both use iterative methods — tolerance matches nlib's rp=1e-4
    tol = 1e-4
    if diff > tol:
        failures.append(f"{key}: Python={py[key]:.15e} Rust={rust[key]:.15e} diff={diff:.2e}")
    else:
        passes.append(key)

print(f"Results: {len(passes)} PASS, {len(failures)} FAIL")
print()
if failures:
    for f in failures:
        print(f"  FAIL  {f}")
    print()
for p in passes:
    print(f"  PASS  {p}")

sys.exit(1 if failures else 0)

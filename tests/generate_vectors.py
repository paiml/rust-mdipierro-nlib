#!/usr/bin/env python3
"""Generate golden test vectors from Di Pierro's formulas.

These vectors are the ground truth: if Rust disagrees with this output,
the Rust implementation has a bug. Regenerate and diff to detect drift.

Usage: python3 tests/generate_vectors.py
"""
import json, math, os

vectors = {}

# === sort ===
input_sort = [38, 27, 43, 3, 9, 82, 10]
vectors["sort"] = {
    "input": input_sort,
    "quicksort": sorted(input_sort),
    "mergesort": sorted(input_sort),
    "heapsort": sorted(input_sort),
}

# === stats ===
x = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]
mu = sum(x) / len(x)
var = sum((v - mu)**2 for v in x) / len(x)
a = [1.0, 2.0, 3.0, 4.0, 5.0]
b = [2.0, 4.0, 6.0, 8.0, 10.0]
ma, mb = sum(a)/len(a), sum(b)/len(b)
cov = sum((ai-ma)*(bi-mb) for ai,bi in zip(a,b)) / len(a)
sa = (sum((v-ma)**2 for v in a)/len(a))**0.5
sb = (sum((v-mb)**2 for v in b)/len(b))**0.5
corr = cov / (sa * sb)
obs = [16.0, 18.0, 16.0, 14.0, 12.0, 12.0]
exp = [16.0, 16.0, 16.0, 16.0, 16.0, 8.0]
chi2 = sum((o-e)**2/e for o,e in zip(obs, exp))
vectors["stats"] = {
    "x": x, "mean": mu, "variance": var,
    "a": a, "b": b, "covariance": cov, "correlation": corr,
    "observed": obs, "expected": exp, "chi_squared": chi2,
}

# === solve ===
vectors["solve"] = {
    "bisection": {"f": "x^2-2", "a": 1.0, "b": 2.0, "expected": math.sqrt(2)},
    "fixed_point": {"g": "cos(x)", "x0": 1.0, "expected": 0.7390851332151607},
}

# === integrate ===
vectors["integrate"] = {
    "trapezoid_sin_0_pi_100": {"expected": 1.9998355038874436},
    "simpson_sin_0_pi_100": {"expected": 2.0000000006764735},
    "simpson_x2_0_1_50": {"expected": 1.0/3.0},
}

# === fourier ===
vectors["fourier"] = {
    "dft_impulse": {
        "input": [[1,0],[0,0],[0,0],[0,0]],
        "output": [[1,0],[1,0],[1,0],[1,0]],
    },
    "dft_dc": {
        "input": [[3,0],[3,0],[3,0],[3,0]],
        "output_0_re": 12.0,
    },
}

# === matrix ===
vectors["matrix"] = {
    "A": [[1,2],[3,4]],
    "determinant": -2.0,
    "transpose": [[1,3],[2,4]],
    "matmul_AB": {
        "A": [[1,2],[3,4]], "B": [[5,6],[7,8]],
        "C": [[19,22],[43,50]],
    },
    "inverse": [[-2.0, 1.0], [1.5, -0.5]],
    "cholesky_input": [[4,2],[2,3]],
    "cholesky_L": [[2.0, 0.0], [1.0, 1.4142135623730951]],
}

# === graph ===
vectors["graph"] = {
    "dijkstra_from_0": {
        "edges": [[0,1,4],[1,2,2],[0,3,1],[1,4,3],[2,5,5],[3,4,6],[4,5,1]],
        "distances": [0.0, 4.0, 6.0, 1.0, 7.0, 8.0],
    },
}

# === random (LCG is deterministic) ===
state = 1
lcg_vals = []
for _ in range(5):
    state = (16807 * state) % 2147483647
    lcg_vals.append(state)
vectors["random"] = {
    "lcg_minstd_seed1_first5": lcg_vals,
}

out_path = os.path.join(os.path.dirname(__file__), "fixtures", "golden_vectors.json")
with open(out_path, "w") as f:
    json.dump(vectors, f, indent=2)
    f.write("\n")
print(f"Generated {len(vectors)} vector groups → {out_path}")

//! Example receipts — contract: `example-receipt-v1.yaml`
//!
//! Every `examples/*.rs` is an entity (ONT-001 §3.7 `extract:json`): run with
//! `--json`, it prints a [`Receipt`] whose checks are the example's own numeric
//! claims, each named after the contract equation it exercises. The tracked copy
//! lives in `evidence/examples/<name>.json`; the example's contract carries a
//! CLOSED SHACL shape that requires every check to hold.
//!
//! R-2 (zero is a decline, never a pass): a receipt with no checks does not pass.

use std::fmt::Write as _;

/// One claim an example makes, and whether it held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// `EX-<EXAMPLE>-<NNN>`.
    pub id: String,
    /// The contract equation the claim exercises.
    pub equation: String,
    pub holds: bool,
}

/// The document an example emits with `--json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub example: String,
    pub checks: Vec<Check>,
}

/// `|got - want| <= tol`, with every operand finite. NaN never holds.
pub fn within(got: f64, want: f64, tol: f64) -> bool {
    got.is_finite() && want.is_finite() && tol.is_finite() && (got - want).abs() <= tol
}

impl Receipt {
    pub fn new(example: &str) -> Self {
        assert!(is_token(example), "receipt: example name `{example}`");
        Self {
            example: example.to_string(),
            checks: Vec::new(),
        }
    }

    /// Record a boolean claim against `equation`.
    pub fn check(&mut self, equation: &str, holds: bool) -> &mut Self {
        assert!(is_token(equation), "receipt: equation name `{equation}`");
        let id = format!(
            "EX-{}-{:03}",
            self.example.to_uppercase().replace('_', ""),
            self.checks.len() + 1
        );
        self.checks.push(Check {
            id,
            equation: equation.to_string(),
            holds,
        });
        self
    }

    /// Record `|got - want| <= tol` against `equation`.
    pub fn close(&mut self, equation: &str, got: f64, want: f64, tol: f64) -> &mut Self {
        self.check(equation, within(got, want, tol))
    }

    /// True iff there is at least one check and every check holds.
    pub fn all_pass(&self) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|c| c.holds)
    }

    /// The canonical JSON form: fixed key order, two-space indent, trailing newline.
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        let _ = writeln!(s, "  \"example\": \"{}\",", self.example);
        let _ = writeln!(s, "  \"source\": \"examples/{}.rs\",", self.example);
        let _ = writeln!(
            s,
            "  \"contract\": \"example-{}-v1\",",
            self.example.replace('_', "-")
        );
        let _ = writeln!(s, "  \"n_checks\": {},", self.checks.len());
        let _ = writeln!(s, "  \"all_pass\": {},", self.all_pass());
        s.push_str("  \"checks\": [");
        for (i, c) in self.checks.iter().enumerate() {
            s.push_str(if i == 0 { "\n" } else { ",\n" });
            let _ = write!(
                s,
                "    {{\"id\": \"{}\", \"equation\": \"{}\", \"holds\": {}}}",
                c.id, c.equation, c.holds
            );
        }
        s.push_str(if self.checks.is_empty() {
            "]\n"
        } else {
            "\n  ]\n"
        });
        s.push_str("}\n");
        s
    }

    /// Print the receipt (`--json`) or a human summary, and return the exit code:
    /// 0 when [`Receipt::all_pass`], 1 otherwise.
    pub fn emit(&self, json: bool) -> i32 {
        if json {
            print!("{}", self.to_json());
        } else {
            println!("\n{} check(s):", self.checks.len());
            for c in &self.checks {
                let mark = if c.holds { "✓" } else { "✗" };
                println!("  {mark} {} {}", c.id, c.equation);
            }
        }
        i32::from(!self.all_pass())
    }
}

/// True when `--json` is among the process arguments.
pub fn json_requested() -> bool {
    wants_json(std::env::args().skip(1))
}

/// True when `--json` is one of `args` (program name already removed).
pub fn wants_json<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter().any(|a| a.as_ref() == "--json")
}

/// Names that go into the JSON unescaped: `[a-z0-9_]+`.
fn is_token(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falsify_rcpt_001_empty_receipt_does_not_pass() {
        let r = Receipt::new("demo");
        assert!(
            !r.all_pass(),
            "zero checks is a decline, never a pass (R-2)"
        );
        assert!(r.to_json().contains("\"all_pass\": false"));
    }

    #[test]
    fn falsify_rcpt_002_one_failed_check_fails_the_receipt() {
        let mut r = Receipt::new("demo");
        r.check("eq_a", true)
            .check("eq_b", false)
            .check("eq_c", true);
        assert!(!r.all_pass());
        assert_eq!(r.emit(true), 1);
    }

    #[test]
    fn falsify_rcpt_003_nan_never_within_tolerance() {
        assert!(!within(f64::NAN, 1.0, 1e-9));
        assert!(!within(1.0, f64::NAN, 1e-9));
        assert!(!within(1.0, 1.0, f64::NAN));
        assert!(!within(f64::INFINITY, f64::INFINITY, 1.0));
        assert!(within(1.0, 1.0 + 1e-12, 1e-9));
        assert!(!within(1.0, 1.0 + 1e-6, 1e-9));
        assert!(within(2.0, 1.0, 1.0), "the bound is inclusive");
    }

    #[test]
    fn falsify_rcpt_004_json_is_canonical() {
        let mut r = Receipt::new("monte_carlo");
        r.check("eq_a", true).close("eq_b", 1.0, 1.0, 0.0);
        let want = "{\n  \"example\": \"monte_carlo\",\n  \"source\": \"examples/monte_carlo.rs\",\n  \"contract\": \"example-monte-carlo-v1\",\n  \"n_checks\": 2,\n  \"all_pass\": true,\n  \"checks\": [\n    {\"id\": \"EX-MONTECARLO-001\", \"equation\": \"eq_a\", \"holds\": true},\n    {\"id\": \"EX-MONTECARLO-002\", \"equation\": \"eq_b\", \"holds\": true}\n  ]\n}\n";
        assert_eq!(r.to_json(), want);
        assert_eq!(r.emit(true), 0);
        assert_eq!(
            Receipt::new("x")
                .to_json()
                .matches("\"checks\": []")
                .count(),
            1
        );
    }

    #[test]
    #[should_panic(expected = "equation name")]
    fn falsify_rcpt_005_unsafe_names_are_refused() {
        Receipt::new("demo").check("bad\"name", true);
    }

    #[test]
    #[should_panic(expected = "example name")]
    fn falsify_rcpt_006_unsafe_example_name_is_refused() {
        let _ = Receipt::new("Bad-Name");
    }

    #[test]
    fn falsify_rcpt_007_only_an_exact_json_flag_selects_json() {
        assert!(wants_json(["--json"]));
        assert!(wants_json(["-q", "--json"]));
        assert!(!wants_json(["--jsonx", "json", "-json", "--JSON"]));
        assert!(!wants_json(Vec::<String>::new()));
        assert!(!json_requested(), "the test harness is not passed --json");
    }
}

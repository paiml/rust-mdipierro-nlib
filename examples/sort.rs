//! Sorting — Di Pierro, *Annotated Algorithms in Python*, §3.5.5, §3.6.1.
//! Contract: `contracts/example-sort-v1.yaml` (entity: this example's `--json` receipt).
//!
//! cargo run --example sort            # human-readable
//! cargo run --example sort -- --json  # the receipt in evidence/examples/sort.json
use nlib::receipt::{Receipt, json_requested};
use nlib::sort::{heapsort, is_permutation, is_sorted, mergesort, quicksort};

/// The input every sort in this example receives.
pub const DATA: [i32; 7] = [38, 27, 43, 3, 9, 82, 10];

/// Every claim this example makes, as a receipt. Each check names the
/// `example-sort-v1` equation it exercises.
pub fn receipt(quiet: bool) -> Receipt {
    let say = |line: String| {
        if !quiet {
            println!("{line}");
        }
    };
    let mut r = Receipt::new("sort");

    let mut q = DATA.to_vec();
    quicksort(&mut q);
    say(format!("quicksort: {q:?}"));
    r.check("quicksort_sorted", is_sorted(&q));
    r.check("quicksort_permutation", is_permutation(&DATA, &q));

    let m = mergesort(&DATA);
    say(format!("mergesort: {m:?}"));
    r.check("mergesort_sorted", is_sorted(&m));
    r.check("mergesort_permutation", is_permutation(&DATA, &m));

    let mut h = DATA.to_vec();
    heapsort(&mut h);
    say(format!("heapsort:  {h:?}"));
    r.check("heapsort_sorted", is_sorted(&h));
    r.check("heapsort_permutation", is_permutation(&DATA, &h));

    r.check("sorts_agree", q == m && m == h);
    say(format!("\nAll three sorts of {DATA:?}"));
    r
}

fn main() {
    let json = json_requested();
    std::process::exit(receipt(json).emit(json));
}

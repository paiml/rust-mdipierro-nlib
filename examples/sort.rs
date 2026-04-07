//! Sorting algorithms — Di Pierro Ch. 3.5
//! cargo run --example sort
use nlib::sort::{quicksort, mergesort, heapsort, is_sorted};

fn main() {
    let data = vec![38, 27, 43, 3, 9, 82, 10];

    let mut q = data.clone();
    quicksort(&mut q);
    println!("quicksort: {q:?}");
    assert!(is_sorted(&q));

    let m = mergesort(&data);
    println!("mergesort: {m:?}");
    assert!(is_sorted(&m));

    let mut h = data.clone();
    heapsort(&mut h);
    println!("heapsort:  {h:?}");
    assert!(is_sorted(&h));

    assert_eq!(q, m);
    assert_eq!(m, h);
    println!("\n✓ All three sorts agree on {:?}", data);
}

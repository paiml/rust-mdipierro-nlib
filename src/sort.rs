//! Sorting algorithms — contract: `sorting-v1.yaml`
//!
//! Di Pierro Ch. 3.5: quicksort, mergesort, heapsort.
//! Postcondition: output is sorted AND is a permutation of input.

/// Quicksort (Lomuto partition). In-place, O(n log n) average.
pub fn quicksort<T: Ord>(a: &mut [T]) {
    if a.len() <= 1 {
        return;
    }
    qs_range(a, 0, a.len() - 1);
}

fn qs_range<T: Ord>(a: &mut [T], lo: usize, hi: usize) {
    if lo >= hi {
        return;
    }
    let p = partition(a, lo, hi);
    if p > 0 {
        qs_range(a, lo, p - 1);
    }
    if p < hi {
        qs_range(a, p + 1, hi);
    }
}

fn partition<T: Ord>(a: &mut [T], lo: usize, hi: usize) -> usize {
    let mut i = lo;
    for j in lo..hi {
        if a[j] <= a[hi] {
            a.swap(i, j);
            i += 1;
        }
    }
    a.swap(i, hi);
    i
}

/// Mergesort. Stable, O(n log n) guaranteed. Returns new Vec.
pub fn mergesort<T: Ord + Clone>(a: &[T]) -> Vec<T> {
    if a.len() <= 1 {
        return a.to_vec();
    }
    let mid = a.len() / 2;
    let left = mergesort(&a[..mid]);
    let right = mergesort(&a[mid..]);
    merge(&left, &right)
}

fn merge<T: Ord + Clone>(l: &[T], r: &[T]) -> Vec<T> {
    let mut out = Vec::with_capacity(l.len() + r.len());
    let (mut i, mut j) = (0, 0);
    while i < l.len() && j < r.len() {
        if l[i] <= r[j] {
            out.push(l[i].clone());
            i += 1;
        } else {
            out.push(r[j].clone());
            j += 1;
        }
    }
    out.extend_from_slice(&l[i..]);
    out.extend_from_slice(&r[j..]);
    out
}

/// Heapsort. In-place, O(n log n) guaranteed.
pub fn heapsort<T: Ord>(a: &mut [T]) {
    let n = a.len();
    if n <= 1 {
        return;
    }
    for i in (0..n / 2).rev() {
        sift_down(a, i, n);
    }
    for i in (1..n).rev() {
        a.swap(0, i);
        sift_down(a, 0, i);
    }
}

fn sift_down<T: Ord>(a: &mut [T], mut root: usize, end: usize) {
    loop {
        let left = 2 * root + 1;
        if left >= end {
            break;
        }
        let right = left + 1;
        let mut max = root;
        if a[left] > a[max] {
            max = left;
        }
        if right < end && a[right] > a[max] {
            max = right;
        }
        if max == root {
            break;
        }
        a.swap(root, max);
        root = max;
    }
}

/// Postcondition helper: check if slice is sorted.
pub fn is_sorted<T: Ord>(a: &[T]) -> bool {
    a.windows(2).all(|w| w[0] <= w[1])
}

/// Postcondition helper: check permutation invariant.
pub fn is_permutation<T: Ord + Clone>(orig: &[T], sorted: &[T]) -> bool {
    if orig.len() != sorted.len() {
        return false;
    }
    let mut a = orig.to_vec();
    let mut b = sorted.to_vec();
    a.sort();
    b.sort();
    a == b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quicksort_empty() {
        let mut a: Vec<i32> = vec![];
        quicksort(&mut a);
        assert!(is_sorted(&a));
    }

    #[test]
    fn quicksort_single() {
        let mut a = vec![42];
        quicksort(&mut a);
        assert!(is_sorted(&a));
    }

    #[test]
    fn quicksort_sorted() {
        let orig = vec![1, 2, 3, 4, 5];
        let mut a = orig.clone();
        quicksort(&mut a);
        assert!(is_sorted(&a));
        assert!(is_permutation(&orig, &a));
    }

    #[test]
    fn quicksort_reverse() {
        let orig = vec![5, 4, 3, 2, 1];
        let mut a = orig.clone();
        quicksort(&mut a);
        assert!(is_sorted(&a));
        assert!(is_permutation(&orig, &a));
    }

    #[test]
    fn quicksort_duplicates() {
        let orig = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        let mut a = orig.clone();
        quicksort(&mut a);
        assert!(is_sorted(&a));
        assert!(is_permutation(&orig, &a));
    }

    #[test]
    fn quicksort_large() {
        let orig: Vec<i32> = (0..1000).rev().collect();
        let mut a = orig.clone();
        quicksort(&mut a);
        assert!(is_sorted(&a));
        assert!(is_permutation(&orig, &a));
    }

    #[test]
    fn mergesort_empty() {
        let r = mergesort::<i32>(&[]);
        assert!(is_sorted(&r));
    }

    #[test]
    fn mergesort_reverse() {
        let a = vec![5, 4, 3, 2, 1];
        let r = mergesort(&a);
        assert!(is_sorted(&r));
        assert!(is_permutation(&a, &r));
    }

    #[test]
    fn mergesort_large() {
        let a: Vec<i32> = (0..1000).rev().collect();
        let r = mergesort(&a);
        assert!(is_sorted(&r));
        assert!(is_permutation(&a, &r));
    }

    #[test]
    fn heapsort_empty() {
        let mut a: Vec<i32> = vec![];
        heapsort(&mut a);
        assert!(is_sorted(&a));
    }

    #[test]
    fn heapsort_reverse() {
        let orig = vec![5, 4, 3, 2, 1];
        let mut a = orig.clone();
        heapsort(&mut a);
        assert!(is_sorted(&a));
        assert!(is_permutation(&orig, &a));
    }

    #[test]
    fn heapsort_duplicates() {
        let orig = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        let mut a = orig.clone();
        heapsort(&mut a);
        assert!(is_sorted(&a));
        assert!(is_permutation(&orig, &a));
    }

    #[test]
    fn heapsort_large() {
        let orig: Vec<i32> = (0..1000).rev().collect();
        let mut a = orig.clone();
        heapsort(&mut a);
        assert!(is_sorted(&a));
        assert!(is_permutation(&orig, &a));
    }

    #[test]
    fn cross_reference_all_sorts() {
        let input = vec![9, 7, 5, 3, 1, 8, 6, 4, 2, 0];
        let expected = mergesort(&input);
        let mut q = input.clone();
        quicksort(&mut q);
        let mut h = input.clone();
        heapsort(&mut h);
        assert_eq!(q, expected);
        assert_eq!(h, expected);
    }

    #[test]
    fn all_same() {
        let orig = vec![7; 100];
        let mut a = orig.clone();
        quicksort(&mut a);
        assert!(is_sorted(&a));
        assert!(is_permutation(&orig, &a));
    }

    #[test]
    fn two_elements() {
        let mut a = vec![2, 1];
        quicksort(&mut a);
        assert_eq!(a, vec![1, 2]);
    }
}

// TODO: Given a vector of integers, leak its heap allocation.
//  Then split the resulting static slice into two halves and
//  sum each half in a separate thread.
//  Hint: check out `Vec::leak`.

use std::thread;

pub fn sum(v: Vec<i32>) -> i32 {
    // Leak the vector's heap allocation
    let v = Vec::leak(v);

    // Split the slice into two halves
    let (left, right) = v.split_at(v.len() / 2);

    // Sum the left half in a separate thread
    let left_handle = thread::spawn(move || left.iter().sum());
    let left_sum: i32 = left_handle.join().unwrap();

    // Sum the right half in the current thread
    let right_handle = thread::spawn(move || right.iter().sum());
    let right_sum: i32 = right_handle.join().unwrap();

    left_sum + right_sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(sum(vec![]), 0);
    }

    #[test]
    fn one() {
        assert_eq!(sum(vec![1]), 1);
    }

    #[test]
    fn five() {
        assert_eq!(sum(vec![1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn nine() {
        assert_eq!(sum(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]), 45);
    }

    #[test]
    fn ten() {
        assert_eq!(sum(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]), 55);
    }
}

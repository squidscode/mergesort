#[allow(unused)]
#[allow(dead_code)]

use rand::{thread_rng, Rng};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::thread;
// use std::thread::JoinHandle;

const SIZE: usize = 10_000_000;
const NUM_THREADS: i32 = 12;
const MIN_SIZE: usize = 10000;
const THREAD_ERROR: &str = "Invalid state!";

fn main() {
    let mut unsorted_vector: Vec<i32> = vec![];
    let mut thread_rng = thread_rng();
    for _ in 0..SIZE {
        unsorted_vector.push(thread_rng.gen_range(1..=SIZE.try_into().unwrap()));
    }
    psort(&mut unsorted_vector[..]);
    // println!("{:?}", unsorted_vector);
}

fn psort(source: &mut [i32]) -> &mut [i32] {
    let counter = Arc::new(Mutex::new(1));
    return psort_internal(source, counter);
}

fn psort_internal<'a>(source: &'a mut [i32], counter: Arc<Mutex<i32>>) -> &'a mut [i32] {
    let n = source.len();
    if n <= 1 { return source; }
    let (lft, rgt) = source.split_at_mut(n / 2);
    let counter_clone = Arc::clone(&counter);
    if psort_should_split(n, &counter) { 
        // println!("Splitting!");
        thread::scope(|sc| {
            sc.spawn(|| { psort_internal(lft,  counter_clone) });
            psort_internal(rgt, counter);
        });
    } else {
        psort_internal(lft, counter_clone);
        psort_internal(rgt, counter);
    }
    
    let new_vec = merge_halves(lft, rgt);
    for i in 0..new_vec.len() {
        source[i] = new_vec[i];
    }
    return source;
}

fn psort_should_split(size: usize, counter: &Arc<Mutex<i32>>) -> bool {
    match counter.try_lock() {
        Ok(mut val) => {
            *val += 1;
            return MIN_SIZE <= size && *val <= NUM_THREADS;
        }
        Err(_) => {
            return false;
        }
    }
}

fn sort(source: &mut [i32]) -> &mut [i32] {
    let mid: usize = source.len() / 2;
    if source.len() <= 1 {
        return source;
    }
    let (left, right) = source.split_at_mut(mid);
    sort(left);
    sort(right);
    let new_vec = merge_halves(left, right);
    for i in 0..new_vec.len() {
        source[i] = new_vec[i];
    }
    return source;
}

fn merge_halves(left: &mut [i32], right: &mut [i32])
    -> Vec<i32> {
    let mut new_vec: Vec<i32> = vec![];
    let (mut l, mut r) = (0, 0);
    let (ln, rn) = (left.len(), right.len());
    while l < ln || r < rn {
        new_vec.push(
            if (l < ln && r < rn && left[l] < right[r])
                || (l < ln && r == rn) {
                l += 1;
                left[l - 1]
            } else {
                r += 1;
                right[r - 1]
            }
        );
    }
    return new_vec;
}
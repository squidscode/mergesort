use rand::{thread_rng, Rng};
use std::sync::Mutex;

use std::thread;
use std::time::Instant;

const SIZE: usize = 50_000_000;
const NUM_THREADS: i32 = 1000;
const USE_RATIO: bool = false;
const PSORT_MERGE_RATIO: f32 = 0.9;
const PSORT_MIN_SIZE: usize = 100_000;
const PMERGE_MIN_SIZE: usize = 100_000;
// const PSORT_MIN_SIZE: usize = SIZE / (NUM_THREADS as f32 * PSORT_MERGE_RATIO) as usize;
// const PMERGE_MIN_SIZE: usize = SIZE / (NUM_THREADS as f32 * (1.0 - PSORT_MERGE_RATIO)) as usize;

fn main() {
    let mut unsorted_vector: Vec<i32> = vec![];
    let mut thread_rng = thread_rng();
    for _ in 0..SIZE {
        unsorted_vector.push(thread_rng.gen_range(1..=SIZE.try_into().unwrap()));
    }
    let mut unsorted_vector_clone: Vec<i32> = unsorted_vector.clone();
    // println!("{:?}", unsorted_vector);
    time("psort", || { psort(&mut unsorted_vector[..]); });
    time("sort", || { sort(&mut unsorted_vector_clone[..]); });
    println!("");
    // println!("{:?}", unsorted_vector);
    let (mut psort_sorted, mut sort_sorted) = (true, true);
    for i in 1..unsorted_vector.len() {
        psort_sorted &= unsorted_vector[i - 1] <= unsorted_vector[i];
        sort_sorted &= unsorted_vector_clone[i - 1] <= unsorted_vector_clone[i];
    }
    println!("psort sorted correctly: {}", psort_sorted);
    println!("sort sorted correctly: {}", sort_sorted);
}

fn time<F>(fn_name: &str, function: F) where F: FnOnce() {
    let bf = Instant::now();
    function();
    println!("{fn_name:} took {} seconds.", bf.elapsed().as_millis() as f32 / 1000.0);
}

fn psort(source: &mut [i32]) -> &mut [i32] {
    let sort_counter = Mutex::new(1);
    let merge_counter = Mutex::new(1);
    if USE_RATIO {
        return psort_internal(source, &sort_counter, &merge_counter);
    } else {
        return psort_internal(source, &sort_counter, &sort_counter)
    }
}

fn psort_internal<'a>(source: &'a mut [i32], sort_counter: &Mutex<i32>, merge_counter: &Mutex<i32>) -> &'a mut [i32] {
    let n = source.len();
    if n <= 1 { return source; }
    let (lft, rgt) = source.split_at_mut(n / 2);
    if psort_should_split(n, sort_counter) { 
        // println!("psort split!");
        thread::scope(|sc| {
            sc.spawn(|| { psort_internal(lft,  sort_counter, merge_counter) });
            psort_internal(rgt, sort_counter, merge_counter);
        });
        *sort_counter.lock().unwrap() -= 1;
    } else {
        psort_internal(lft, sort_counter, merge_counter);
        psort_internal(rgt, sort_counter, merge_counter);
    }
    let mut new_vec: Vec<i32> = Vec::new();
    new_vec.resize(lft.len() + rgt.len(), 0);
    pmerge_halves(lft, rgt, &mut new_vec[..], merge_counter);
    source[..new_vec.len()].copy_from_slice(&new_vec[..]);
    source
}

fn psort_should_split(size: usize, counter: &Mutex<i32>) -> bool {
    match counter.try_lock() {
        Ok(mut val) => {
            let max_threads = if USE_RATIO {NUM_THREADS as f32 * PSORT_MERGE_RATIO} else {NUM_THREADS as f32};
            let allow = PSORT_MIN_SIZE <= size && *val < (max_threads).round() as i32;
            if allow {
                *val += 1;
            }
            allow
        }
        Err(_) => {
            false
        }
    }
}

fn pmerge_halves(left: &[i32], right: &[i32], dest: &mut [i32], counter: &Mutex<i32>) {
    if !pmerge_should_run(left.len() + right.len(), counter) {
        return merge_halves(left, right, dest);
    }
    let (bigger, smaller) = if left.len() < right.len() { 
        (right, left) 
    } else { 
        (left, right) 
    };
    if bigger.len() <= 1 {
        return merge_halves(left, right, dest);
    }
    let bmid = bigger.len() / 2;
    let smid = match smaller.binary_search(&bigger[bmid]) {
        Ok(smid) => smid,
        Err(smid) => smid
    };
    let (lb, rb) = (&bigger[..bmid], &bigger[bmid..]);
    let (ls, rs) = (&smaller[..smid], &smaller[smid..]);
    let (ldest, rdest) = dest.split_at_mut(bmid + smid);
    // println!("pmerge_halves split!");
    thread::scope(|sc| {
        sc.spawn(|| { pmerge_halves(lb, ls, ldest, counter) });
        pmerge_halves(rb, rs, rdest, counter);
    });
    *counter.lock().unwrap() -= 1;
    // println!("# of pmerge threads: {}", *cl)
}

fn pmerge_should_run(size: usize, counter: &Mutex<i32>) -> bool {
    match counter.try_lock() {
        Ok(mut val) => {
            let max_threads = if USE_RATIO {NUM_THREADS as f32 * (1.0 - PSORT_MERGE_RATIO)} else {NUM_THREADS as f32};
            let allow = PMERGE_MIN_SIZE <= size && *val < (max_threads).round() as i32;
            if allow {
                *val += 1;
            }
            allow
        }
        Err(_) => {
            false
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
    let mut new_vec: Vec<i32> = Vec::new();
    new_vec.resize(left.len() + right.len(), 0);
    merge_halves(left, right, &mut new_vec[..]);
    source[..new_vec.len()].copy_from_slice(&new_vec[..]);
    source
}

fn merge_halves(left: &[i32], right: &[i32], dest: &mut [i32]) {
    let (mut l, mut r) = (0, 0);
    let mut ind: usize = 0;
    let (ln, rn) = (left.len(), right.len());
    while l < ln || r < rn {
        dest[ind] = if (l < ln && r < rn && left[l] < right[r]) || (l < ln && r == rn) {
            l += 1;
            left[l - 1]
        } else {
            r += 1;
            right[r - 1]
        };
        ind += 1;
    }
}
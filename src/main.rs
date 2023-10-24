use rand::{random, thread_rng, Rng};

const SIZE: usize = 1000;

fn main() {
    let mut unsorted_vector: Vec<i32> = vec![0; SIZE];
    let mut thread_rng = thread_rng();
    for x in unsorted_vector.iter_mut() {
        *x = thread_rng.gen_range(1..=100000);
    }
    println!("{:?}", sort(unsorted_vector.as_mut_slice()));
}

fn sort(source: &mut [i32]) -> &mut [i32] {
    let mid: usize = source.len() / 2;
    if source.len() <= 1 {
        return source;
    }
    sort(&mut source[..mid]);
    sort(&mut source[mid..]);
    merge_halves(source);
    return source;
}

fn merge_halves(source: &mut [i32]) {
    let mid: usize = source.len() / 2;
    let mut new_vec: Vec<i32> = vec![];
    let mut l: usize = 0; let mut r: usize = mid;
    while l < mid || r < source.len() {
        if l < mid && r < source.len() {
            if source[l] < source[r] {
                new_vec.push(source[l]);
                l += 1;
            } else {
                new_vec.push(source[r]);
                r += 1;
            }
        } else {
            if l < mid {
                new_vec.push(source[l]); 
                l += 1;
            } else {
                new_vec.push(source[r]);
                r += 1;
            }
        }
    }
    for i in 0..source.len() {
        source[i] = new_vec[i];
    }
}
#![feature(is_sorted)]

use std::{f32::consts::PI, time::{Duration, Instant}, fs::File, io::Write};
use blase::random::{random_u32, random_usize};

fn radix_sort (mut result: Vec<u32>) -> Vec<u32> {
    #[inline(always)]
    fn test_bit (v: u32, idx: u32) -> bool {
        ((v >> idx) & 1) == 1
    }

    for i in 0..u32::BITS {
        let mut left = Vec::with_capacity(result.len());
        let mut right = Vec::with_capacity(result.len());

        for v in result.into_iter() {
            if test_bit(v, i) {
                right.push(v);
            } else {
                left.push(v);
            }
        }

        left.extend(right);
        result = left;
    }

    result
}

/*
fn radix_float (v: Vec<f32>) {
    let by_parts = v.into_iter().map(|x| {
        let bits = x.to_bits();
        let exp = ((bits >> 23) & 0xff) as u8;
        let mant = bits & 0x7FFFFF;
        (x, exp, mant)
    }).collect::<Vec<_>>();

    let radixed = radix_sort(by_parts, 1, |x| x.2);
    let radixed = radix_sort(radixed, 1, |x| x.1);
    todo!()
}
*/

fn pseudo_radix (v: &mut [u32]) {
    #[inline(always)]
    fn test_bit (v: u32, idx: u32) -> bool {
        ((v >> idx) & 1) == 1
    }

    for i in 0..u32::BITS {
        let mut left_most_one = None;

        for j in 0..v.len() {
            if test_bit(v[j], i) {
                // one
                if left_most_one.is_none() {
                    left_most_one = Some(j);
                }
            } else {
                // zero
                if let Some(ref mut lmo) = left_most_one {
                    let mut k = j;
                    while k > *lmo {
                        v.swap(k, k-1);
                        k -= 1;
                    }

                    *lmo += 1;
                }
            }
        }
    }
}

fn quicksort<T: Ord> (v: &mut [T]) {
    if v.len() < 2 {
        return;
    }

    let pivot = unsafe {
        v.get_unchecked(v.len() - 1)
    };

    let mut left_ptr = 0;
    let mut right_ptr = v.len() - 1;

    loop {
        while &v[left_ptr] <= pivot && left_ptr < right_ptr {
            left_ptr += 1;
        }

        while &v[right_ptr] >= pivot && left_ptr < right_ptr {
            right_ptr -= 1;
        }

        if left_ptr == right_ptr {
            v.swap(left_ptr, v.len() - 1);
            quicksort(&mut v[..left_ptr]);
            quicksort(&mut v[left_ptr + 1..]);
            break;
        } else {
            unsafe {
                core::ptr::swap(
                    std::ptr::addr_of!(v[left_ptr]) as *mut T,
                    std::ptr::addr_of!(v[right_ptr]) as *mut T
                )
            }
        }
    }
}

fn quicksort_random<T: Ord> (v: &mut [T]) {
    if v.len() < 2 {
        return;
    }

    let pivot = random_usize(..v.len());
    v.swap(pivot, v.len() - 1);

    let pivot = unsafe {
        v.get_unchecked(v.len() - 1)
    };

    let mut left_ptr = 0;
    let mut right_ptr = v.len() - 1;

    loop {
        while &v[left_ptr] <= pivot && left_ptr < right_ptr {
            left_ptr += 1;
        }

        while &v[right_ptr] >= pivot && left_ptr < right_ptr {
            right_ptr -= 1;
        }

        if left_ptr == right_ptr {
            v.swap(left_ptr, v.len() - 1);
            quicksort(&mut v[..left_ptr]);
            quicksort(&mut v[left_ptr + 1..]);
            break;
        } else {
            unsafe {
                core::ptr::swap(
                    std::ptr::addr_of!(v[left_ptr]) as *mut T,
                    std::ptr::addr_of!(v[right_ptr]) as *mut T
                )
            }
        }
    }
}

const EPOCHS : u128 = 100;
const ITERS : usize = 100;

#[test]
fn test () {
    let mut v = [1, 8, 3, 9, 4, 5, 7];
    quicksort(&mut v);
    println!("{:?}", v);
}

#[test]
fn bench () {
    let mut out = File::options()
        .create(true)
        .write(true)
        .open("radix_bench_5.csv")
        .unwrap();

    out.write_fmt(format_args!("VALUES,RUST STABLE,RUST UNSTABLE,RADIX,QUICKSORT,QUICKSORT RANDOM,PSEUDO\n")).unwrap();

    for i in 1..=ITERS {
        let len = 10 * i;

        let stable = bench_test(|mut x| {
            x.sort();
            x
        }, len);
        let unstable = bench_test(|mut x| {
            x.sort_unstable();
            x
        }, len);
        let normal = bench_test(radix_sort, len);
        let quicksort = bench_test(|mut x| {
            quicksort(&mut x);
            x
        }, len);
        let quicksort_rand = bench_test(|mut x| {
            quicksort_random(&mut x);
            x
        }, len);
        let new = bench_test(|mut x| {
            pseudo_radix(&mut x);
            x
        }, len);

        out.write_fmt(format_args!("{len},{stable},{unstable},{normal},{quicksort},{quicksort_rand},{new}\n")).unwrap();

        let pct = 100f32 * (i as f32) / (ITERS as f32);
        println!("{pct:.2}%")
    }

    out.flush().unwrap();
}

fn bench_test<F: Fn(Vec<u32>) -> Vec<u32>> (f: F, len: usize) -> u128 {
    let mut duration = Duration::default();

    for _ in 0..EPOCHS {
        let v = random_vec(len);
        let n = Instant::now();
        let v = f(v);
        let dur = n.elapsed();
        assert!(v.is_sorted());
        duration += dur;
    }

    duration.as_nanos() / EPOCHS
}

#[inline]
fn random_vec (len: usize) -> Vec<u32> {
    let mut v = Vec::with_capacity(len);
    for _ in 0..len {
        v.push(random_u32(..15));
    }
    v
}
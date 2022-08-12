#![feature(bench_black_box, is_sorted, sort_floats)]

use std::{time::Duration, fs::File, io::Write};
use blase::{random::{Random}, vec::EucVec};
use blaze_rs::prelude::*;

const EPOCHS : u128 = 100;
const ITERS : usize = 100;

type Number = f32;
const MIN : Number = 0f32;
const MAX : Number = 1f32;

#[global_context]
static CTX : SimpleContext = SimpleContext::default();

#[test]
fn bench () {
    let mut rng = Random::new(None).unwrap();
    let mut file = File::options()
        .create(true)
        .write(true)
        .open("bench_sort.csv")
        .unwrap();

    file.write_fmt(format_args!("VALUES,CPU,GPU\n")).unwrap();

    for i in 1..=ITERS {
        let len = 100 * i;

        let buffer = rng.next_f32(len, MIN..=MAX, true, false)
            .map(EucVec::from_buffer)
            .unwrap();
        let slice = buffer.map(.., EMPTY).unwrap().wait_unwrap();
        
        let cpu = cpu_test(&slice);
        let gpu = gpu_test(&buffer);
        file.write_fmt(format_args!("{len},{cpu},{gpu}\n")).unwrap();
        
        let pct = 100f32 * (i as f32) / (ITERS as f32);
        println!("{pct:.2}%");
    }

    file.flush().unwrap();
}


fn cpu_test (v: &[Number]) -> u128 {
    let mut result = Duration::default();
    for _ in 0..EPOCHS {
        let mut vec = v.to_vec();
        let now = std::time::Instant::now();
        vec.sort_floats();
        let dur = now.elapsed();
        assert!(vec.is_sorted());
        result += dur;
    }

    result.as_nanos() / EPOCHS
}

fn gpu_test (v: &EucVec<Number>) -> u128 {
    let mut result = Duration::default();
    for _ in 0..EPOCHS {
        let now = std::time::Instant::now();
        let sorted = v.sort(EMPTY).unwrap().wait_unwrap();
        let dur = now.elapsed();
        assert!(sorted.read(.., EMPTY).unwrap().wait_unwrap().is_sorted());
        result += dur;
    }

    result.as_nanos() / EPOCHS
}
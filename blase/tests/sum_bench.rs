use std::{time::{Duration, Instant}, ops::{Deref}, fs::File, io::Write};
use blase::{Real, utils::DerefCell, work_group_size, random::Random};
use blaze_proc::global_context;
use blaze_rs::prelude::*;

type Number = f32;

fn full_cpu_sum_st (lhs: &[Number]) -> (Number, Duration) {
    let now = Instant::now();
    let v = lhs.into_iter().sum::<Number>();
    let dur = now.elapsed();
    (v, dur)
}

fn full_cpu_sum_mt (lhs: &[Number]) -> (Number, Duration) {
    let threads = std::thread::available_parallelism().unwrap();
    let now = Instant::now();
    
    let v = std::thread::scope(|s| {
        let handles = lhs
            .chunks(threads.get())
            .map(|x| s.spawn(move || x.into_iter().sum::<Number>()))
            .collect::<Vec<_>>();
        
        let mut v = 0 as Number;
        for handle in handles {
            v += handle.join().unwrap();
        }

        return v
    });

    let dur = now.elapsed();
    (v, dur)
}

/*
RAYON MT (suprisingly slow)
let v = lhs.into_par_iter()
        .copied()
        .sum();
*/

fn full_gpu_sum (lhs: &Buffer<Number>) -> Result<(Number, Duration)> {
    let len = lhs.len()?;
    let wgs = work_group_size(len);

    let result = Buffer::<Number>::new_uninit(1, MemAccess::default(), false).map(DerefCell)?;
    let evt = unsafe {
        <Number as Real>::vec_program().vec_sum(len, lhs, result, [wgs], None, EMPTY)?
    };

    let ((_, out), kernel_dur) : (_, Duration) = evt.wait_with_duration()?;
    unsafe {
        let out : Buffer<Number> = out.0.assume_init();
        let (v, read_dur) = out.read(.., EMPTY)?.wait_with_duration()?;
        Ok((v[0], kernel_dur + read_dur))
    }
}

#[inline(always)]
fn gpu_cpu_sum_mt (lhs: &Buffer<Number>) -> Result<(Number, Duration)> {
    gpu_cpu_sum(lhs, full_cpu_sum_mt)
}

#[inline(always)]
fn gpu_cpu_sum_st (lhs: &Buffer<Number>) -> Result<(Number, Duration)> {
    gpu_cpu_sum(lhs, full_cpu_sum_st)
}

fn gpu_cpu_sum<F: Fn(&[Number]) -> (Number, Duration)> (lhs: &Buffer<Number>, f: F) -> Result<(Number, Duration)> {
    let len = lhs.len()?;
    let wgs = work_group_size(len);

    let result = Buffer::<Number>::new_uninit(wgs, MemAccess::default(), false).map(DerefCell)?;
    let evt = unsafe {
        <Number as Real>::vec_program().sum_cpu(len, lhs, result, [wgs], None, EMPTY)?
    };

    let ((_, out), kernel_dur) : (_, Duration) = evt.wait_with_duration()?;
    unsafe {
        let out : Buffer<Number> = out.0.assume_init();
        let (v, read_dur) = out.map(.., EMPTY)?.wait_with_duration()?;
        let (v, cpu_dur) = f(v.deref());
        Ok((v, kernel_dur + read_dur + cpu_dur))
    }
}

const EPOCHS : u128 = 100;

#[global_context]
static CTX : SimpleContext = SimpleContext::default();

#[test]
fn bench () -> Result<()> {
    let mut rng = Random::new(None)?;
    let mut file = File::options()
        .create(true)
        .write(true)
        .open("sum_bench.csv")
        .unwrap();

    file.write_all(b"VALUES, FULL GPU, FULL CPU, FULL CPU MT, GPU-CPU, GPU-CPU MT\n").unwrap();

    for i in 1..=250 {
        let len = 100 * i;
        let buffer = rng.next_f32(len, 0f32..1f32, true, false)?;
        let slice = buffer.map(.., EMPTY)?.wait()?;

        let gpu_time = bench_mean_gpu(&buffer, full_gpu_sum)?;
        let cpu_time_st = bench_mean_cpu(&slice, full_cpu_sum_st);
        let cpu_time_mt = bench_mean_cpu(&slice, full_cpu_sum_mt);
        let gpu_cpu_time = bench_mean_gpu(&buffer, gpu_cpu_sum_st)?;
        let gpu_cpu_time_mt = bench_mean_gpu(&buffer, gpu_cpu_sum_mt)?;

        file.write_all(
            format!(
                "{len},{gpu_time},{cpu_time_st},{cpu_time_mt},{gpu_cpu_time},{gpu_cpu_time_mt}\n",
            ).as_bytes()
        ).unwrap();
    }

    file.flush().unwrap();
    Ok(())
}

fn bench_mean_cpu<F: Fn(&[Number]) -> (Number, Duration)> (buffer: &[Number], f: F) -> u128 {
    let mut result = Duration::default();

    for _ in 0..EPOCHS {
        let (_, dur) = f(buffer);
        result += dur;
    }

    result.as_nanos() / EPOCHS
}


fn bench_mean_gpu<F: Fn(&Buffer<Number>) -> Result<(Number, Duration)>> (buffer: &Buffer<Number>, f: F) -> Result<u128> {
    let mut result = Duration::default();

    for _ in 0..EPOCHS {
        let (_, dur) = f(buffer)?;
        result += dur;
    }

    Ok(result.as_nanos() / EPOCHS)
}

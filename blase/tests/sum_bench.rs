use std::{time::{Duration, Instant}, ops::{Deref}, fs::File, io::Write, mem::MaybeUninit};
use blase::{Real, utils::DerefCell, work_group_size, random::Random};
use blaze_proc::{global_context, blaze};
use blaze_rs::prelude::*;
use once_cell::sync::Lazy;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

type Number = u32;
static BLAST : Lazy<BlastSum> = Lazy::new(|| BlastSum::new(None).unwrap());

#[blaze(BlastSum)]
#[link = include_str!("../src/opencl/blast_sum.cl")]
extern "C" {
    #[link_name = "Xasum"]
    fn xasum (n: i32, x: *const Number, x_offset: i32, x_inc: i32, output: *mut MaybeUninit<Number>);
    #[link_name = "XasumEpilogue"]
    fn xasum_epilogue (input: *const MaybeUninit<Number>, asum: *mut MaybeUninit<Number>, assum_offset: i32);
}

fn full_cpu_sum_st (lhs: &[Number]) -> (Number, Duration) {
    let now = Instant::now();
    let v = lhs.into_iter().sum::<Number>();
    let dur = now.elapsed();
    (v, dur)
}

fn full_cpu_sum_mt (lhs: &[Number]) -> (Number, Duration) {
    let now = Instant::now();
    let v = lhs.into_par_iter()
        .copied()
        .sum();
    let dur = now.elapsed();
    (v, dur)
}

// CURRENTLY NOT WORKING CORRECTLY
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

fn full_gpu_blast_sum (lhs: &Buffer<Number>) -> Result<(Number, Duration)> {
    const WGS1 : usize = 64;
    const WGS2 : usize = 64;

    let n = lhs.len()?;
    let temp_size = 2 * WGS2;
    let mut temp_buffer = Buffer::<Number>::new_uninit(2 * WGS2, MemAccess::default(), false)?;
    
    let evt = unsafe {
        BLAST.xasum(n as i32, lhs, 0, 1, &mut temp_buffer, [WGS1 * temp_size], [WGS1], EMPTY)?
    };

    let (_, kernel_dur) : (_, Duration) = evt.wait_with_duration()?;
    let mut asum = Buffer::new_uninit(1, MemAccess::WRITE_ONLY, false)?;

    let evt2 = unsafe {
        BLAST.xasum_epilogue(&mut temp_buffer, &mut asum, 0, [WGS2], [WGS2], EMPTY)?
    };

    let (_, epilogue_dur) = evt2.wait_with_duration()?;
    let (v, read_dur) = unsafe {
        asum.assume_init().read(.., EMPTY)?.wait_with_duration()?
    };

    Ok((v[0], kernel_dur + epilogue_dur + read_dur))
}

fn full_gpu_atomic_sum (lhs: &Buffer<Number>) -> Result<(Number, Duration)> {
    let len = lhs.len()?;
    let wgs = work_group_size(len);

    let result = Buffer::<Number>::new_uninit(1, MemAccess::default(), false).map(DerefCell)?;
    let evt = unsafe {
        <Number as Real>::vec_program().sum_atomic(len, lhs, result, [wgs], None, EMPTY)?
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

/*
    TODO OPTIMIZE FULL GPU ATOMIC
*/

#[test]
fn bench () -> Result<()> {
    let mut rng = Random::new(None)?;
    let mut file = File::options()
        .create(true)
        .write(true)
        .open("sum_bench_u32_ext.csv")
        .unwrap();

    file.write_all(b"VALUES, FULL GPU, FULL CPU, FULL CPU MT, GPU-CPU, GPU-CPU MT, FULL GPU ATOMIC\n").unwrap();

    for i in 1..=500 {
        let len = 100 * i;
        let max_value = u32::MAX / (len as u32);
        assert!(max_value > 0);

        let buffer = rng.next_u32(len, ..=max_value, true, false)?;
        let slice = buffer.map(.., EMPTY)?.wait()?;

        let gpu_time = bench_mean_gpu(&buffer, full_gpu_blast_sum)?;
        //let gpu_time = bench_mean_gpu(&buffer, full_gpu_sum)?;
        let gpu_atomic_time = bench_mean_gpu(&buffer, full_gpu_atomic_sum)?;
        let cpu_time_st = bench_mean_cpu(&slice, full_cpu_sum_st);
        let cpu_time_mt = bench_mean_cpu(&slice, full_cpu_sum_mt);
        let gpu_cpu_time = bench_mean_gpu(&buffer, gpu_cpu_sum_st)?;
        let gpu_cpu_time_mt = bench_mean_gpu(&buffer, gpu_cpu_sum_mt)?;

        file.write_all(
            format!(
                "{len},{gpu_time},{cpu_time_st},{cpu_time_mt},{gpu_cpu_time},{gpu_cpu_time_mt},{gpu_atomic_time}\n",
            ).as_bytes()
        ).unwrap();

        let pct = 100f32 * (i as f32) / 500f32;
        println!("{pct:.2}%"); 
    }

    file.flush().unwrap();
    Ok(())
}

#[test]
fn test_atomic () -> Result<()> {
    const LEN : usize = 100;
    const MAX_VALUE : u32 = u32::MAX / (LEN as u32);
    let mut rng = Random::new(None)?;

    let buffer = rng.next_u32(LEN, ..=MAX_VALUE, true, false)?;
    let slice = buffer.map(.., EMPTY)?.wait()?;

    let (cpu, _) = full_cpu_sum_st(&slice);
    let (gpu, _) = full_gpu_blast_sum(&buffer)?;

    assert_eq!(cpu, gpu);
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

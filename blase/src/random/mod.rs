use blaze_proc::{blaze, docfg};
use blaze_rs::prelude::*;
use std::{mem::MaybeUninit, num::NonZeroUsize};
use crate::define_usize;
use std::ops::{RangeBounds, Bound};

flat_mod!(cpu);

#[blaze(RandomProgram)]
#[link = generate_program(include_str!("../opencl/random.cl"))]
extern "C" {
    fn random_uchar (n: usize, seed: *mut u64, out: *mut MaybeUninit<u8>, origin: u8, delta: u8);
    fn random_ushort (n: usize, seed: *mut u64, out: *mut MaybeUninit<u16>, origin: u16, delta: u16);
    fn random_uint (n: usize, seed: *mut u64, out: *mut MaybeUninit<u32>, origin: u32, delta: u32);
    fn random_ulong (n: usize, seed: *mut u64, out: *mut MaybeUninit<u64>, origin: u64, delta: u64);

    fn random_char (n: usize, seed: *mut u64, out: *mut MaybeUninit<i8>, origin: i8, delta: i8);
    fn random_short (n: usize, seed: *mut u64, out: *mut MaybeUninit<i16>, origin: i16, delta: i16);
    fn random_int (n: usize, seed: *mut u64, out: *mut MaybeUninit<i32>, origin: i32, delta: i32);
    fn random_long (n: usize, seed: *mut u64, out: *mut MaybeUninit<i64>, origin: i64, delta: i64);

    #[cfg(feature = "half")]
    fn random_half (n: usize, seed: *mut u64, out: *mut MaybeUninit<::half::f16>, origin: ::half::f16, delta: ::half::f16);
    fn random_float (n: usize, seed: *mut u64, out: *mut MaybeUninit<f32>, origin: f32, delta: f32);
    #[cfg(feature = "double")]
    fn random_double (n: usize, seed: *mut u64, out: *mut MaybeUninit<f64>, origin: f64, delta: f64);

    fn loop_random_uchar (n: usize, seed: *mut u64, out: *mut MaybeUninit<u8>, origin: u8, bound: u8);
    fn loop_random_ushort (n: usize, seed: *mut u64, out: *mut MaybeUninit<u16>, origin: u16, bound: u16);
    fn loop_random_uint (n: usize, seed: *mut u64, out: *mut MaybeUninit<u32>, origin: u32, bound: u32);
    fn loop_random_ulong (n: usize, seed: *mut u64, out: *mut MaybeUninit<u64>, origin: u64, bound: u64);

    fn loop_random_char (n: usize, seed: *mut u64, out: *mut MaybeUninit<i8>, origin: i8, bound: i8);
    fn loop_random_short (n: usize, seed: *mut u64, out: *mut MaybeUninit<i16>, origin: i16, bound: i16);
    fn loop_random_int (n: usize, seed: *mut u64, out: *mut MaybeUninit<i32>, origin: i32, bound: i32);
    fn loop_random_long (n: usize, seed: *mut u64, out: *mut MaybeUninit<i64>, origin: i64, bound: i64);
}

macro_rules! impl_int {
    ($($t:ty as $f:ident => ($k:ident, $loop:ident)),+) => {
        $(
            #[inline(always)]
            pub fn $f (&mut self, len: usize, range: impl RangeBounds<$t>, readable: bool, alloc: bool) -> Result<Buffer<$t, C>> {
                #[inline]
                fn ranged_by_loop<C: Context + Clone> (this: &mut Random<C>, origin: $t, bound: $t, readable: bool, alloc: bool, len: usize, wgs: usize) -> Result<Buffer<$t, C>> {
                    let mut result = Buffer::<$t, C>::new_uninit_in(
                        this.seeds.context().clone(), len, MemAccess::new(readable, true), alloc
                    )?;

                    unsafe {
                        let _ = this.program.$loop(
                            len,
                            &mut this.seeds, &mut result,
                            origin, bound,
                            [wgs], None, None
                        )?;
            
                        return Ok(result.assume_init())
                    }
                }
                
                let wgs = self.seeds.len()?.min(len);

                let origin = match range.start_bound() {
                    Bound::Included(x) => *x,
                    Bound::Excluded(x) => match x.checked_add(1) {
                        Some(x) => x,
                        None => return Err(Error::new(ErrorKind::InvalidValue, "Range start bound is too large"))
                    },
                    Bound::Unbounded => <$t>::MIN
                };

                let bound = match range.end_bound() {
                    Bound::Included(x) => match x.checked_add(1) {
                        Some(x) => x,
                        None => return ranged_by_loop(self, origin, <$t>::MAX, readable, alloc, len, wgs)
                    },
                    Bound::Excluded(x) => *x,
                    Bound::Unbounded => <$t>::MAX
                };

                if let Some(delta) = bound.checked_sub(origin) {
                    let mut result = Buffer::new_uninit_in(
                        self.seeds.context().clone(), len, MemAccess::new(readable, true), alloc
                    )?;
            
                    unsafe {
                        let _ = self.program.$k(
                            len,
                            &mut self.seeds, &mut result,
                            origin, delta,
                            [wgs], None, None
                        )?;
            
                        return Ok(result.assume_init())
                    }
                } else if <$t>::MIN != 0 {
                    return match bound.checked_sub(1) {
                        Some(bound) => ranged_by_loop(self, origin, bound, readable, alloc, len, wgs),
                        None => Err(Error::new(ErrorKind::InvalidValue, "Range end bound is too small"))
                    }
                }

                Err(Error::new(ErrorKind::InvalidValue, "Invalid range"))
            }
        )+
    };
}

pub struct Random<C: Context = Global> {
    seeds: Buffer<u64, C>,
    program: RandomProgram<C>
}

impl Random {
    #[inline(always)]
    pub fn new (seed_count: Option<NonZeroUsize>) -> Result<Self> {
        Self::new_in(Global, seed_count)
    }
}

impl<C: Context + Clone> Random<C> {
    #[inline(always)]
    pub fn new_in (ctx: C, seed_count: Option<NonZeroUsize>) -> Result<Self> where C: 'static {
        Self::with_rng_in(ctx, &thread_rng(), seed_count)
    }

    pub fn with_rng_in (ctx: C, rng: &LocalRandom, seed_count: Option<NonZeroUsize>) -> Result<Self> where C: 'static {
        let seed_count = match seed_count {
            Some(x) => x.get(),
            None => {
                let mut max_wgs : Option<NonZeroUsize> = None;
                for queue in ctx.queues() {
                    let device = queue.device()?;
                    let wgs = device.max_work_group_size()?;

                    max_wgs = match max_wgs {
                        Some(x) => Some(x.min(wgs)),
                        None => Some(wgs)
                    }
                }

                if let Some(max_wgs) = max_wgs {
                    max_wgs.get()
                } else {
                    return Err(Error::new(ErrorKind::InvalidDevice, "No devices found"));
                }
            }
        };
        
        let mut seeds = Buffer::<u64, _>::new_uninit_in(ctx.clone(), seed_count, MemAccess::default(), false)?;
        let mut map = seeds.map_mut_blocking(.., None)?;

        for i in 0..seed_count {
            unsafe {
                map.get_unchecked_mut(i).write(rng.next_u64(..));
            }
        }

        drop(map);
        unsafe {
            Self::with_seeds(seeds.assume_init())
        }
    }

    #[inline(always)]
    pub fn with_seeds (seeds: Buffer<u64, C>) -> Result<Self> {
        let program = RandomProgram::new_in(seeds.context().clone(), None)?;
        Ok(Self { program, seeds })
    }

    impl_int! {
        u8 as next_u8_blocking => (random_uchar_blocking, loop_random_uchar_blocking),
        u16 as next_u16_blocking => (random_ushort_blocking, loop_random_ushort_blocking),
        u32 as next_u32_blocking => (random_uint_blocking, loop_random_uint_blocking),
        u64 as next_u64_blocking => (random_ulong_blocking, loop_random_ulong_blocking),
        i8 as next_i8_blocking => (random_char_blocking, loop_random_char_blocking),
        i16 as next_i16_blocking => (random_short_blocking, loop_random_short_blocking),
        i32 as next_i32_blocking => (random_int_blocking, loop_random_int_blocking),
        i64 as next_i64_blocking => (random_long_blocking, loop_random_long_blocking)
    }

    #[docfg(feature = "half")]
    pub fn next_f16_blocking (&mut self, len: usize, range: impl RangeBounds<::half::f16>, readable: bool, alloc: bool) -> Result<Buffer<::half::f16, C>> {
        let wgs = self.seeds.len()?.min(len);

        let origin = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x + ::half::f16::EPSILON,
            Bound::Unbounded => ::half::f16::MIN,
        };

        let bound = match range.end_bound() {
            Bound::Included(x) => *x + ::half::f16::EPSILON,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => ::half::f16::MAX,
        };

        let mut result = Buffer::new_uninit_in(
            self.seeds.context().clone(), len, MemAccess::new(readable, true), alloc
        )?;

        unsafe {
            let _ = self.program.random_half_blocking(
                len,
                &mut self.seeds, &mut result,
                origin, bound - origin,
                [wgs], None, None
            )?;

            return Ok(result.assume_init())
        }
    }

    pub fn next_f32_blocking (&mut self, len: usize, range: impl RangeBounds<f32>, readable: bool, alloc: bool) -> Result<Buffer<f32, C>> {
        let wgs = self.seeds.len()?.min(len);

        let origin = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x + f32::EPSILON,
            Bound::Unbounded => f32::MIN,
        };

        let bound = match range.end_bound() {
            Bound::Included(x) => *x + f32::EPSILON,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => f32::MAX,
        };

        let mut result = Buffer::new_uninit_in(
            self.seeds.context().clone(), len, MemAccess::new(readable, true), alloc
        )?;

        unsafe {
            let _ = self.program.random_float_blocking(
                len,
                &mut self.seeds, &mut result,
                origin, bound - origin,
                [wgs], None, None
            )?;

            return Ok(result.assume_init())
        }
    }

    #[docfg(feature = "double")]
    pub fn next_f64_blocking (&mut self, len: usize, range: impl RangeBounds<f64>, readable: bool, alloc: bool) -> Result<Buffer<f64, C>> {
        let wgs = self.seeds.len()?.min(len);

        let origin = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x + f64::EPSILON,
            Bound::Unbounded => f64::MIN,
        };

        let bound = match range.end_bound() {
            Bound::Included(x) => *x + f64::EPSILON,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => f64::MAX,
        };

        let mut result = Buffer::new_uninit_in(
            self.seeds.context().clone(), len, MemAccess::new(readable, true), alloc
        )?;

        unsafe {
            let _ = self.program.random_double_blocking(
                len,
                &mut self.seeds, &mut result,
                origin, bound - origin,
                [wgs], None, None
            )?;

            return Ok(result.assume_init())
        }
    }
}

#[inline(always)]
fn generate_program (src: &str) -> String {
    cfg_if::cfg_if! {
        if #[cfg(all(feature = "half", feature = "double"))] {
            const EXTENSIONS : &'static str = "
                #pragma OPENCL EXTENSION cl_khr_fp64: enable
                #pragma OPENCL EXTENSION cl_khr_fp16: enable
                #define HALF true
                #define DOUBLE true
            ";
        } else if #[cfg(feature = "half")] {
            const EXTENSIONS : &'static str = "
                #pragma OPENCL EXTENSION cl_khr_fp16: enable
                #define HALF true
            ";
        } else if #[cfg(feature = "double")] {
            const EXTENSIONS : &'static str = "
                #pragma OPENCL EXTENSION cl_khr_fp64: enable
                #define DOUBLE true
            ";
        } else {
            const EXTENSIONS : &'static str = "";
        }
    }

    format!(
        "{}{}{src}",
        EXTENSIONS,
        define_usize()
    )
}

#[cfg(test)]
mod test {
    use blaze_rs::prelude::*;
    use super::Random;

    #[global_context]
    static CTX : SimpleContext = SimpleContext::default();

    #[test]
    fn add () -> Result<()> {
        let mut rng = Random::new(None)?;
        
        let test = rng.next_f32_blocking(100, 0.0..1., true, false)?;
        let test = test.read_blocking(.., None)?;
        
        println!("{test:?}");

        Ok(())
    }
}
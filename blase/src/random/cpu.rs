use std::{cell::{UnsafeCell}, sync::atomic::{AtomicU64, Ordering}, num::Wrapping, time::{SystemTime, UNIX_EPOCH}, mem::transmute};
use std::ops::{RangeBounds, Bound};

use blaze_proc::docfg;

const MULTIPLIER : Wrapping<u64> = Wrapping(0x5DEECE66Du64);
const ADDEND : Wrapping<u64> = Wrapping(0xBu64);
const MASK : Wrapping<u64> = Wrapping((1u64 << 48) - 1);
static UNIQUIFIER : AtomicU64 = AtomicU64::new(8682522807148012u64);

thread_local! {
    static THREAD_RNG : std::rc::Rc<LocalRandom> = std::rc::Rc::new(LocalRandom::new());
}

macro_rules! impl_int {
    ($($t:ty as $f:ident),+) => {
        $(
            #[doc = concat!("Generates a random `", stringify!($t), "` within the specified range.")]
            #[inline]
            pub fn $f (&self, range: impl RangeBounds<$t>) -> $t {
                #[inline(always)]
                fn ranged_by_loop (this: &LocalRandom, range: impl RangeBounds<$t>) -> $t {
                    loop {
                        let v = this.next::<{<$t>::BITS as usize}>() as $t;
                        if range.contains(&v) { return v; }
                    }
                }

                let origin = match range.start_bound() {
                    Bound::Included(start) => *start,
                    Bound::Excluded(start) => match start.checked_add(1) {
                        Some(x) => x,
                        None => return ranged_by_loop(self, range)
                    },
                    Bound::Unbounded => <$t>::MIN,
                };

                let bound = match range.end_bound() {
                    Bound::Included(end) => match end.checked_add(1) {
                        Some(x) => x,
                        None => return ranged_by_loop(self, range)
                    },
                    Bound::Excluded(end) => *end,
                    Bound::Unbounded => return ranged_by_loop(self, range),
                };

                if let Some(delta) = bound.checked_sub(origin) {
                    let v;

                    // compiler should optimize this away
                    if <$t>::MIN == 0 {
                        // unsigned int
                        v = self.next::<{<$t>::BITS as usize}>() as $t;
                    } else {
                        // signed int
                        v = self.next::<{<$t>::BITS as usize - 1}>() as $t;
                    }

                    return (v % delta) + origin;
                }

                ranged_by_loop(self, range)
            }
        )+
    };
}

/// A thread-unsafe non-cryptographic random number generator.
/// Translation of Java's `Random` class.
#[repr(transparent)]
pub struct LocalRandom {
    seed: UnsafeCell<Wrapping<u64>>
}

impl LocalRandom {
    pub fn new () -> Self {
        unsafe {
            let current = UNIQUIFIER.fetch_update(
                Ordering::AcqRel, 
                Ordering::Acquire,
                |x| Some(x.wrapping_mul(181783497276652981u64))
            ).unwrap_unchecked();
    
            let nanos = Wrapping(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_unchecked().as_nanos() as u64);
            Self::from_wrapping_seed(nanos ^ Wrapping(current))
        }
    }

    #[inline(always)]
    pub fn from_seed (seed: u64) -> Self {
        Self::from_wrapping_seed(Wrapping(seed))
    }

    #[inline(always)]
    pub fn from_wrapping_seed (seed: Wrapping<u64>) -> Self {
        Self {
            seed: UnsafeCell::new((seed ^ MULTIPLIER) & MASK)
        }
    }

    /// Generates a random `bool`.
    #[inline(always)]
    pub fn next_bool (&self) -> bool {
        self.next::<1>() & 1 == 1
    }

    impl_int! {
        u8 as next_u8,
        u16 as next_u16,
        u32 as next_u32,
        i8 as next_i8,
        i16 as next_i16,
        i32 as next_i32
    }

    /// Generates a random `u64` within the specified range.
    #[inline]
    pub fn next_u64 (&self, range: impl RangeBounds<u64>) -> u64 {
        #[inline(always)]
        fn ranged_by_loop (this: &LocalRandom, range: impl RangeBounds<u64>) -> u64 {
            loop {
                let v = ((this.next::<32>() as u64) << 32) | (this.next::<32>() as u64);
                if range.contains(&v) { return v; }
            }
        }

        let origin = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => match start.checked_add(1) {
                Some(x) => x,
                None => return ranged_by_loop(self, range)
            },
            Bound::Unbounded => <u64>::MIN,
        };

        let bound = match range.end_bound() {
            Bound::Included(end) => match end.checked_add(1) {
                Some(x) => x,
                None => return ranged_by_loop(self, range)
            },
            Bound::Excluded(end) => *end,
            Bound::Unbounded => return ranged_by_loop(self, range),
        };

        if let Some(delta) = bound.checked_sub(origin) {
            let v = ((self.next::<32>() as u64) << 32) | (self.next::<32>() as u64);
            return (v % delta) + origin;
        }

        ranged_by_loop(self, range)
    }

    /// Generates a random `i64` within the specified range.
    #[inline]
    pub fn next_i64 (&self, range: impl RangeBounds<i64>) -> i64 {
        #[inline(always)]
        fn ranged_by_loop (this: &LocalRandom, range: impl RangeBounds<i64>) -> i64 {
            loop {
                let v = ((this.next::<32>() as i64) << 32) | (this.next::<32>() as i64);
                if range.contains(&v) { return v; }
            }
        }

        let origin = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => match start.checked_add(1) {
                Some(x) => x,
                None => return ranged_by_loop(self, range)
            },
            Bound::Unbounded => <i64>::MIN,
        };

        let bound = match range.end_bound() {
            Bound::Included(end) => match end.checked_add(1) {
                Some(x) => x,
                None => return ranged_by_loop(self, range)
            },
            Bound::Excluded(end) => *end,
            Bound::Unbounded => return ranged_by_loop(self, range),
        };

        if let Some(delta) = bound.checked_sub(origin) {
            let v = ((self.next::<31>() as i64) << 32) | (self.next::<32>() as i64);
            return (v % delta) + origin;
        }

        ranged_by_loop(self, range)
    }

    /// Generates a random `u128` within the specified range.
    #[inline]
    pub fn next_u128 (&self, range: impl RangeBounds<u128>) -> u128 {
        #[inline(always)]
        fn ranged_by_loop (this: &LocalRandom, range: impl RangeBounds<u128>) -> u128 {
            loop {
                let v = 
                    ((this.next::<32>() as u128) << 96) |
                    ((this.next::<32>() as u128) << 64) |
                    ((this.next::<32>() as u128) << 32) |
                    (this.next::<32>() as u128);

                if range.contains(&v) { return v; }
            }
        }

        let origin = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => match start.checked_add(1) {
                Some(x) => x,
                None => return ranged_by_loop(self, range)
            },
            Bound::Unbounded => <u128>::MIN,
        };

        let bound = match range.end_bound() {
            Bound::Included(end) => match end.checked_add(1) {
                Some(x) => x,
                None => return ranged_by_loop(self, range)
            },
            Bound::Excluded(end) => *end,
            Bound::Unbounded => return ranged_by_loop(self, range),
        };

        if let Some(delta) = bound.checked_sub(origin) {
            let v = 
                ((self.next::<32>() as u128) << 96) |
                ((self.next::<32>() as u128) << 64) |
                ((self.next::<32>() as u128) << 32) |
                (self.next::<32>() as u128);

            return (v % delta) + origin;
        }

        ranged_by_loop(self, range)
    }

    /// Generates a random `i128` within the specified range.
    #[inline]
    pub fn next_i128 (&self, range: impl RangeBounds<i128>) -> i128 {
        #[inline(always)]
        fn ranged_by_loop (this: &LocalRandom, range: impl RangeBounds<i128>) -> i128 {
            loop {
                let v = 
                    ((this.next::<32>() as i128) << 96) |
                    ((this.next::<32>() as i128) << 64) |
                    ((this.next::<32>() as i128) << 32) |
                    (this.next::<32>() as i128);

                if range.contains(&v) { return v; }
            }
        }

        let origin = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => match start.checked_add(1) {
                Some(x) => x,
                None => return ranged_by_loop(self, range)
            },
            Bound::Unbounded => <i128>::MIN,
        };

        let bound = match range.end_bound() {
            Bound::Included(end) => match end.checked_add(1) {
                Some(x) => x,
                None => return ranged_by_loop(self, range)
            },
            Bound::Excluded(end) => *end,
            Bound::Unbounded => return ranged_by_loop(self, range),
        };

        if let Some(delta) = bound.checked_sub(origin) {
            let v = 
                ((self.next::<31>() as i128) << 96) |
                ((self.next::<32>() as i128) << 64) |
                ((self.next::<32>() as i128) << 32) |
                (self.next::<32>() as i128);
                
            return (v % delta) + origin;
        }

        ranged_by_loop(self, range)
    }

    cfg_if::cfg_if! {
        if #[cfg(target_pointer_width = "8")] {
            /// Generates a random `usize` within the specified range.
            #[inline(always)]
            pub fn next_usize (&self, range: impl RangeBounds<usize>) -> usize {
                self.next_u8(CastedRange(range)) as usize
            }

            /// Generates a random `isize` within the specified range.
            #[inline(always)]
            pub fn next_isize (&self, range: impl RangeBounds<isize>) -> isize {
                self.next_i8(CastedRange(range)) as isize
            }
        } else if #[cfg(target_pointer_width = "16")] {
            /// Generates a random `usize` within the specified range.
            #[inline(always)]
            pub fn next_usize (&self, range: impl RangeBounds<usize>) -> usize {
                self.next_u16(CastedRange(range)) as usize
            }

            /// Generates a random `isize` within the specified range.
            #[inline(always)]
            pub fn next_isize (&self, range: impl RangeBounds<isize>) -> isize {
                self.next_i16(CastedRange(range)) as isize
            }
        } else if #[cfg(target_pointer_width = "32")] {
            /// Generates a random `usize` within the specified range.
            #[inline(always)]
            pub fn next_usize (&self, range: impl RangeBounds<usize>) -> usize {
                self.next_u32(CastedRange(range)) as usize
            }

            /// Generates a random `isize` within the specified range.
            #[inline(always)]
            pub fn next_isize (&self, range: impl RangeBounds<isize>) -> isize {
                self.next_i32(CastedRange(range)) as isize
            }
        } else if #[cfg(target_pointer_width = "64")] {
            /// Generates a random `usize` within the specified range.
            #[inline(always)]
            pub fn next_usize (&self, range: impl RangeBounds<usize>) -> usize {
                self.next_u64(CastedRange(range)) as usize
            }

            /// Generates a random `isize` within the specified range.
            #[inline(always)]
            pub fn next_isize (&self, range: impl RangeBounds<isize>) -> isize {
                self.next_i64(CastedRange(range)) as isize
            }
        } else if #[cfg(target_pointer_width = "128")] {
            /// Generates a random `usize` within the specified range.
            #[inline(always)]
            pub fn next_usize (&self, range: impl RangeBounds<usize>) -> usize {
                self.next_u128(CastedRange(range)) as usize
            }

            /// Generates a random `isize` within the specified range.
            #[inline(always)]
            pub fn next_isize (&self, range: impl RangeBounds<isize>) -> isize {
                self.next_i128(CastedRange(range)) as isize
            }
        } else {
            compile_error!("Unsupported target pointer width");
        }
    }

    /// Generates a random `f16` within the specified range.
    #[docfg(feature = "half")]
    #[inline(always)]
    pub fn next_f16 (&self, range: impl RangeBounds<::half::f16>) -> ::half::f16 {
        const FLOAT_UNIT_INC : f32 = ((1u32 << 11) - 1) as f32;
        const FLOAT_UNIT_EXC : f32 = (1u32 << 11) as f32;

        let origin = match range.start_bound() {
            Bound::Included(x) => x.to_f32(),
            Bound::Excluded(x) => x.to_f32() + ::half::f16::EPSILON.to_f32_const(),
            Bound::Unbounded => ::half::f16::MIN.to_f32_const(),
        };

        let (bound, unit) = match range.end_bound() {
            Bound::Included(x) => (x.to_f32(), FLOAT_UNIT_INC),
            Bound::Excluded(x) => (x.to_f32(), FLOAT_UNIT_EXC),
            Bound::Unbounded => (::half::f16::MAX.to_f32_const(), FLOAT_UNIT_INC),
        };

        let delta = bound - origin;
        let v = (self.next::<11>() as f32) / unit;
        ::half::f16::from_f32(origin + delta * v)
    }

    /// Generates a random `f32` within the specified range.
    #[inline(always)]
    pub fn next_f32 (&self, range: impl RangeBounds<f32>) -> f32 {
        const FLOAT_UNIT_INC : f32 = ((1u32 << 24) - 1) as f32;
        const FLOAT_UNIT_EXC : f32 = (1u32 << 24) as f32;

        let origin = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x + f32::EPSILON,
            Bound::Unbounded => f32::MIN,
        };

        let (bound, unit) = match range.end_bound() {
            Bound::Included(x) => (*x, FLOAT_UNIT_INC),
            Bound::Excluded(x) => (*x, FLOAT_UNIT_EXC),
            Bound::Unbounded => (f32::MAX, FLOAT_UNIT_INC),
        };

        let delta = bound - origin;
        let v = (self.next::<24>() as f32) / unit;
        origin + delta * v
    }

    /// Generates a random `f64` within the specified range.
    #[inline(always)]
    pub fn next_f64 (&self, range: impl RangeBounds<f64>) -> f64 {
        const FLOAT_UNIT_INC : f64 = ((1u64 << 53) - 1) as f64;
        const FLOAT_UNIT_EXC : f64 = (1u64 << 53) as f64;

        let origin = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => *x + f64::EPSILON,
            Bound::Unbounded => f64::MIN,
        };

        let (bound, unit) = match range.end_bound() {
            Bound::Included(x) => (*x, FLOAT_UNIT_INC),
            Bound::Excluded(x) => (*x, FLOAT_UNIT_EXC),
            Bound::Unbounded => (f64::MAX, FLOAT_UNIT_INC),
        };

        let delta = bound - origin;
        let v = ((self.next::<26>() as u64) << 27) | (self.next::<27>() as u64);
        origin + delta * (v as f64) / unit
    }

    #[inline]
    fn next<const BITS: usize> (&self) -> u32 {
        // SAFETY: This structure cannot be shared across threads, so we are the only thread accessing this variable.
        let seed = unsafe { &mut *self.seed.get() };
        let next = (*seed * MULTIPLIER + ADDEND) & MASK;
        *seed = next;
        return (next >> (48 - BITS)).0 as u32;
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "8")] {
        type UsizeInner = u8;
        type IsizeInner = i8;
    } else if #[cfg(target_pointer_width = "16")] {
        type UsizeInner = u16;
        type IsizeInner = i16;
    } else if #[cfg(target_pointer_width = "32")] {
        type UsizeInner = u32;
        type IsizeInner = i32;
    } else if #[cfg(target_pointer_width = "64")] {
        type UsizeInner = u64;
        type IsizeInner = i64;
    } else if #[cfg(target_pointer_width = "128")] {
        type UsizeInner = u128;
        type IsizeInner = i128;
    } else {
        compile_error!("Unsupported target pointer width");
    }
}

struct CastedRange<T>(T);

impl<T: RangeBounds<usize>> RangeBounds<UsizeInner> for CastedRange<T> {
    #[inline(always)]
    fn start_bound(&self) -> Bound<&UsizeInner> {
        match self.0.start_bound() {
            Bound::Included(x) => unsafe {
                Bound::Included(transmute(x))
            },

            Bound::Excluded(x) => unsafe {
                Bound::Excluded(transmute(x))
            },

            Bound::Unbounded => Bound::Unbounded,
        }
    }

    #[inline(always)]
    fn end_bound(&self) -> Bound<&UsizeInner> {
        match self.0.end_bound() {
            Bound::Included(x) => unsafe {
                Bound::Included(transmute(x))
            },

            Bound::Excluded(x) => unsafe {
                Bound::Excluded(transmute(x))
            },

            Bound::Unbounded => Bound::Unbounded,
        }
    }
}

impl<T: RangeBounds<isize>> RangeBounds<IsizeInner> for CastedRange<T> {
    #[inline(always)]
    fn start_bound(&self) -> Bound<&IsizeInner> {
        match self.0.start_bound() {
            Bound::Included(x) => unsafe {
                Bound::Included(transmute(x))
            },

            Bound::Excluded(x) => unsafe {
                Bound::Excluded(transmute(x))
            },

            Bound::Unbounded => Bound::Unbounded,
        }
    }

    #[inline(always)]
    fn end_bound(&self) -> Bound<&IsizeInner> {
        match self.0.end_bound() {
            Bound::Included(x) => unsafe {
                Bound::Included(transmute(x))
            },

            Bound::Excluded(x) => unsafe {
                Bound::Excluded(transmute(x))
            },

            Bound::Unbounded => Bound::Unbounded,
        }
    }
}

#[cfg(test)]
mod test {
    use super::LocalRandom;

    #[test]
    fn local () {
        const EPOCHS : usize = 100_000;
        const MIN : usize = 2;
        const MAX : usize = 10;
        const LEN : usize = (MAX - MIN) as usize;

        let random = LocalRandom::new();
        let mut results = [0usize; LEN];

        for _ in 0..EPOCHS {
            let v = random.next_usize(MIN..MAX);
            results[(v - MIN) as usize] += 1;
        }

        let sum = results.iter().map(|x| *x as f32).sum::<f32>();

        for i in MIN..MAX {
            let pct = 100.0 * (results[(i - MIN) as usize] as f32) / sum;
            println!("{i}: {} ({pct:.2} %)", results[(i - MIN) as usize]);
        }
    }

    #[test]
    fn neg () {
        let random = LocalRandom::new();
        
        for _ in 0..10 {
            let v = random.next_f64(1.0..=5.0);
            if v < 1.0 || v > 5.0 {
                panic!("{}", v);
            }

            println!("{v}")
        }
    }
}

#[inline(always)]
pub fn thread_rng () -> std::rc::Rc<LocalRandom> {
    THREAD_RNG.with(|x| x.clone())
}

macro_rules! impl_static {
    ($($t:ty as $fn:ident => $f:ident),+) => {
        $(
            #[doc = concat!("Generates a random value of type `", stringify!($t), "` within the specified range.")]
            #[inline(always)]
            pub fn $fn (range: impl RangeBounds<$t>) -> $t {
                thread_rng().$f(range)
            }
        )+
    };
}

impl_static! {
    u8 as random_u8 => next_u8,
    u16 as random_u16 => next_u16,
    u32 as random_u32 => next_u32,
    u64 as random_u64 => next_u64,
    u128 as random_u128 => next_u128,
    usize as random_usize => next_usize,

    i8 as random_i8 => next_i8,
    i16 as random_i16 => next_i16,
    i32 as random_i32 => next_i32,
    i64 as random_i64 => next_i64,
    i128 as random_i128 => next_i128,

    f32 as random_f32 => next_f32,
    f64 as random_f64 => next_f64
}

/// Generates a random `bool`.
#[inline(always)]
pub fn random_bool () -> bool {
    thread_rng().next_bool()
}

/// Generates a random value of type `f16` within the specified range.
#[docfg(feature = "half")]
#[inline(always)]
pub fn random_f16 (range: impl RangeBounds<::half::f16>) -> ::half::f16 {
    thread_rng().next_f16(range)
}
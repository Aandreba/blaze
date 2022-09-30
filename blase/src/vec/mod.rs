// pub mod dot;
// pub mod sum;
// pub mod cmp;
// pub mod ord;
// pub mod sort;

flat_mod!(utils);

use std::{mem::{MaybeUninit, transmute}, ops::*, fmt::Debug, cmp::Ordering};
use bitvec::prelude::BitBox;
use blaze_rs::{prelude::*, buffer::{KernelPointer}, WaitList, wait_list_from_ref, event::FlagEvent};
use crate::{Real, work_group_size, utils::{change_lifetime_mut, change_lifetime}, vec::events::VecEq};
use self::events::{BinaryEvent, LaneEqEvent, LaneEq, LaneCmpEvent, LaneTotalCmpEvent, EqEvent};
use blaze_proc::docfg;

pub mod program {
    use blaze_proc::blaze;
    use crate::{Real, include_prog, max_work_group_size};
    use std::mem::MaybeUninit;

    #[blaze(VectorProgram<T: Real>)]
    #[link = generate_vec_program::<T>()]
    pub extern "C" {
        // Vertical ops
        pub(super) fn add (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
        pub(super) fn sub (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<T>);
        pub(super) fn scal (n: usize, alpha: T, rhs: *const T, out: *mut MaybeUninit<T>);
        pub(super) fn scal_down (n: usize, lhs: *const T, alpha: T, out: *mut MaybeUninit<T>);
        pub(super) fn scal_down_inv (n: usize, alpha: T, rhs: *const T, out: *mut MaybeUninit<T>);
        #[link_name = "eq"]
        pub(super) fn vec_eq (n: usize, lhs: *const T, rhs: *const T, out: *mut u32);
        #[link_name = "total_eq"]
        pub(super) fn vec_total_eq (n: usize, lhs: *const T, rhs: *const T, out: *mut u32);
        #[link_name = "cmp"]
        pub(super) fn vec_cmp_eq (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<u32>);
        #[link_name = "ord"]
        pub(super) fn vec_cmp_ord (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<i8>);
        #[link_name = "partial_ord"]
        pub(super) fn vec_cmp_partial_ord (n: usize, lhs: *const T, rhs: *const T, out: *mut MaybeUninit<i8>);

        // Horizontal ops
        #[link_name = "Xasum"]
        pub(super) fn xasum (n: i32, x: *const T, output: *mut MaybeUninit<T>);
        #[link_name = "XasumEpilogue"]
        pub(super) fn xasum_epilogue (input: *const MaybeUninit<T>, asum: *mut MaybeUninit<T>);
        #[link_name = "Xdot"]
        pub(super) fn xdot (n: i32, x: *const T, y: *const T, output: *mut MaybeUninit<T>);
        
        // Sort
        #[link_name = "Sort_BitonicMergesortStart"]
        pub(super) fn sort_start (desc: bool, in_array: *const T, out_array: *mut MaybeUninit<T>);
        #[link_name = "Sort_BitonicMergesortLocal"]
        pub(super) fn sort_local (desc: bool, data: *mut MaybeUninit<T>, size: usize, blocksize: usize, stride: usize);
        #[link_name = "Sort_BitonicMergesortGlobal"]
        pub(super) fn sort_global (desc: bool, data: *mut MaybeUninit<T>, size: usize, blocksize: usize, stride: usize);
    }

    fn generate_vec_program<T: Real> () -> String {
        // https://downloads.ti.com/mctools/esd/docs/opencl/execution/kernels-workgroups-workitems.html
        format!(
            "
                {0}
                #define WGS1 {1}
                #define WGS2 {1}
                {2}
                {3}
                {4}
            ",
            include_prog::<T>(include_str!("../opencl/vec.cl")),
            usize::max(max_work_group_size().get() / 2, 2),
            include_str!("../opencl/blast_sum.cl"),
            include_str!("../opencl/blast_dot.cl"),
            include_str!("../opencl/sort.cl"),
        )
    }
}

mod events {
    use blaze_rs::{event::{Consumer, consumer::Map}, buffer::events::{BufferRead, BufferGet}};
    use super::*;

    #[derive(Debug)]
    pub enum VecEq<'a> {
        Host (bool),
        Device (BufferGet<'a, u32>)
    }

    impl Consumer for VecEq<'_> {
        type Output = bool;

        #[inline(always)]
        fn consume (self) -> Result<Self::Output> {
            return match self {
                Self::Host(x) => Ok(x),
                Self::Device(x) => Ok(x.consume()? == 1)
            }
        }
    }

    #[derive(Debug)]
    pub struct LaneEq<'a> {
        pub(super) len: usize,
        pub(super) read: BufferRead<'a, u32>
    }

    impl<'a> Consumer for LaneEq<'a> {
        type Output = (BitBox<u32>, usize);

        #[inline(always)]
        fn consume (self) -> Result<Self::Output> {
            let result = self.read.consume()?;
            Ok((BitBox::from_boxed_slice(result.into_boxed_slice()), self.len))
        }
    }

    pub type LaneCmp<'a> = Map<Vec<i8>, BufferRead<'a, i8>, fn(Vec<i8>) -> Vec<Option<Ordering>>>;
    pub type LaneTotalCmp<'a> = Map<Vec<i8>, BufferRead<'a, i8>, fn(Vec<i8>) -> Vec<Ordering>>;

    /// Event for binary operations
    pub type BinaryEvent<'a, T> = Event<Binary<'a, T>>;
    pub type EqEvent<'a> = Event<VecEq<'a>>;
    pub type LaneEqEvent<'a> = Event<LaneEq<'a>>;
    pub type LaneCmpEvent<'a> = Event<LaneCmp<'a>>;
    pub type LaneTotalCmpEvent<'a> = Event<LaneTotalCmp<'a>>;
}

/// Euclidian vector
#[repr(transparent)]
pub struct EucVec<T: Copy> {
    inner: Buffer<T>
}

impl<T: Copy> EucVec<T> {
    /// Creates a new vector
    pub fn new (v: &[T], alloc: bool) -> Result<Self> {
        let inner = Buffer::new(v, MemAccess::default(), alloc)?;
        Ok(Self { inner })
    }

    /// Creates a new uninitialized vector
    #[inline(always)]
    pub fn new_uninit (len: usize, alloc: bool) -> Result<EucVec<MaybeUninit<T>>> {
        let inner = Buffer::<T, _>::new_uninit(len, MemAccess::default(), alloc)?;
        Ok(EucVec { inner })
    }

    /// Turns a buffer into a vector. This is a zero-cost operation.
    #[inline(always)]
    pub const fn from_buffer (inner: Buffer<T>) -> Self {
        Self { inner }
    }

    /// Turns the vector into a buffer. This is a zero-cost operation.
    #[inline(always)]
    pub fn into_buffer (self) -> Buffer<T> {
        self.inner
    }

    #[inline(always)]
    pub unsafe fn transmute<U: Copy> (self) -> EucVec<U> {
        EucVec { inner: self.inner.transmute() }
    }
}

impl<T: Copy> EucVec<MaybeUninit<T>> {
    #[inline(always)]
    pub unsafe fn assume_init (self) -> EucVec<T> {
        self.transmute()
    }
}

impl<T: Real> EucVec<T> {
    #[inline(always)]
    pub fn add<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: &'env Self, wait: WaitList) -> Result<BinaryEvent<'scope, T>> {
        let len = self.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorKind::InvalidBufferSize, "vectors of diferent sizes provided"))
        }

        let mut result = EucVec::<T>::new_uninit(len, false)?;
        unsafe {
            let event = T::vec_program().add(scope, len, self, other, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            return Ok(Event::map_consumer(event, |_| Binary::new(result)));
        }
    }

    #[inline(always)]
    pub fn add_blocking (&self, other: &Self, wait: WaitList) -> Result<Self> {
        let len = self.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorKind::InvalidBufferSize, "vectors of diferent sizes provided"))
        }

        let mut result = EucVec::new_uninit(len, false)?;
        unsafe {
            T::vec_program().add_blocking(len, self, other, &mut result, [work_group_size(len)], None, wait)?;
            return Ok(result.assume_init());
        }
    }

    #[inline(always)]
    pub fn sub<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: &'env Self, wait: WaitList) -> Result<BinaryEvent<'scope, T>> {
        let len = self.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorKind::InvalidBufferSize, "vectors of diferent sizes provided"))
        }

        let mut result = EucVec::<T>::new_uninit(len, false)?;
        unsafe {
            let event = T::vec_program().sub(scope, len, self, other, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            return Ok(Event::map_consumer(event, |_| Binary::new(result)));
        }
    }

    #[inline(always)]
    pub fn sub_blocking (&self, other: &Self, wait: WaitList) -> Result<Self> {
        let len = self.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorKind::InvalidBufferSize, "vectors of diferent sizes provided"))
        }

        let mut result = EucVec::new_uninit(len, false)?;
        unsafe {
            T::vec_program().sub_blocking(len, self, other, &mut result, [work_group_size(len)], None, wait)?;
            return Ok(result.assume_init());
        }
    }

    #[inline(always)]
    pub fn upscale<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: T, wait: WaitList) -> Result<BinaryEvent<'scope, T>> {
        let len = self.len()?;
        let mut result = EucVec::<T>::new_uninit(len, false)?;

        unsafe {
            let event = T::vec_program().scal(scope, len, other, self, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            return Ok(Event::map_consumer(event, |_| Binary::new(result)));
        }
    }

    #[inline(always)]
    pub fn upscale_blocking (&self, other: T, wait: WaitList) -> Result<Self> {
        let len = self.len()?;
        let mut result = EucVec::new_uninit(len, false)?;
        unsafe {
            T::vec_program().scal_blocking(len, other, self, &mut result, [work_group_size(len)], None, wait)?;
            return Ok(result.assume_init());
        }
    }

    #[inline(always)]
    pub fn downscale<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: T, wait: WaitList) -> Result<BinaryEvent<'scope, T>> {
        let len = self.len()?;
        let mut result = EucVec::<T>::new_uninit(len, false)?;

        unsafe {
            let event = T::vec_program().scal_down(scope, len, self, other, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            return Ok(Event::map_consumer(event, |_| Binary::new(result)));
        }
    }

    #[inline(always)]
    pub fn downscale_blocking (&self, other: T, wait: WaitList) -> Result<Self> {
        let len = self.len()?;
        let mut result = EucVec::new_uninit(len, false)?;
        unsafe {
            T::vec_program().scal_down_blocking(len, self, other, &mut result, [work_group_size(len)], None, wait)?;
            return Ok(result.assume_init());
        }
    }

    #[inline(always)]
    pub fn downscale_inv<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: T, wait: WaitList) -> Result<BinaryEvent<'scope, T>> {
        let len = self.len()?;
        let mut result = EucVec::<T>::new_uninit(len, false)?;

        unsafe {
            let event = T::vec_program().scal_down_inv(scope, len, other, self, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            return Ok(Event::map_consumer(event, |_| Binary::new(result)));
        }
    }

    #[inline(always)]
    pub fn downscale_inv_blocking (&self, other: T, wait: WaitList) -> Result<Self> {
        let len = self.len()?;
        let mut result = EucVec::new_uninit(len, false)?;
        unsafe {
            T::vec_program().scal_down_inv_blocking(len, other, self, &mut result, [work_group_size(len)], None, wait)?;
            return Ok(result.assume_init());
        }
    }
}

// Compare and ordering
impl<T: Real> EucVec<T> {
    /// Compares if both vectors are equal, blocking the current thread until the operation has completed.
    pub fn eq_blocking (&self, other: &Self, wait: WaitList) -> Result<bool> {
        if self.inner.eq_buffer(&other.inner) { return Ok(true) }

        let len = self.len()?;
        if len != other.len()? { return Ok(false) }

        let mut result = blaze_rs::buffer![1u32]?;
        unsafe {
            T::vec_program().vec_eq_blocking(len, self, other, &mut result, [work_group_size(len)], None, wait)?;
            return Ok(result.get_blocking(0, None)? == 1);
        }
    }

    /// Compares if both vectors are equal.
    pub fn eq<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: &'env Self, wait: WaitList) -> Result<EqEvent<'scope>> {
        macro_rules! completed {
            () => {{
                let flag = FlagEvent::new()?;
                flag.try_mark(None)?;
                flag.subscribe()
            }};
        }
        
        if self.inner.eq_buffer(&other.inner) {
            return Ok(Event::new(completed!(), VecEq::Host(true)))
        }

        let len = self.len()?;
        if len != other.len()? {
            return Ok(Event::new(completed!(), VecEq::Host(false)))
        }

        let mut result = blaze_rs::buffer![1u32]?;
        unsafe {
            let evt = T::vec_program().vec_eq(scope, len, self, other, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            let get = change_lifetime(&result).get(scope, 0, wait_list_from_ref(&evt))?;
            return Ok(Event::map_consumer(get, VecEq::Device))
        }
    }

    /// Compares if both vectors are equal, blocking the current thread until the operation has completed.
    pub fn total_eq_blocking (&self, other: &Self, wait: WaitList) -> Result<bool> {
        if self.inner.eq_buffer(&other.inner) { return Ok(true) }

        let len = self.len()?;
        if len != other.len()? { return Ok(false) }

        let mut result = blaze_rs::buffer![1u32]?;
        unsafe {
            T::vec_program().vec_total_eq_blocking(len, self, other, &mut result, [work_group_size(len)], None, wait)?;
            return Ok(result.get_blocking(0, None)? == 1);
        }
    }

    /// Compares if both vectors are equal.
    pub fn total_eq<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: &'env Self, wait: WaitList) -> Result<EqEvent<'scope>> {
        macro_rules! completed {
            () => {{
                let flag = FlagEvent::new()?;
                flag.try_mark(None)?;
                flag.subscribe()
            }};
        }
        
        if self.inner.eq_buffer(&other.inner) {
            return Ok(Event::new(completed!(), VecEq::Host(true)))
        }

        let len = self.len()?;
        if len != other.len()? {
            return Ok(Event::new(completed!(), VecEq::Host(false)))
        }

        let mut result = blaze_rs::buffer![1u32]?;
        unsafe {
            let evt = T::vec_program().vec_total_eq(scope, len, self, other, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            let get = change_lifetime(&result).get(scope, 0, wait_list_from_ref(&evt))?;
            return Ok(Event::map_consumer(get, VecEq::Device))
        }
    }

    pub fn lane_eq<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: &'env Self, wait: WaitList) -> Result<LaneEqEvent<'scope>> {
        let len = self.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorKind::InvalidBufferSize, "vectors of diferent sizes provided"))
        }

        let result_len = len.div_ceil(u32::BITS as usize);
        let mut result = Buffer::<u32>::new_uninit(result_len, MemAccess::WRITE_ONLY, false)?;
        
        unsafe {
            let evt = T::vec_program().vec_cmp_eq(scope, len, self, other, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            let result = change_lifetime(&result.assume_init()).read(scope, .., wait_list_from_ref(&evt))?;
            return Ok(Event::map_consumer(result, |read| LaneEq { len, read }));
        }
    }

    pub fn lane_eq_blocking (&self, other: &Self, wait: WaitList) -> Result<(BitBox<u32>, usize)> {
        let len = self.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorKind::InvalidBufferSize, "vectors of diferent sizes provided"))
        }

        let result_len = len.div_ceil(u32::BITS as usize);
        let mut result = Buffer::<u32>::new_uninit(result_len, MemAccess::WRITE_ONLY, false)?;
        
        unsafe {
            T::vec_program().vec_cmp_eq_blocking(len, self, other, &mut result, [work_group_size(len)], None, wait)?;
            let result = result.assume_init().read_blocking(.., None)?;
            let bbox = BitBox::from_boxed_slice(result.into_boxed_slice());
            return Ok((bbox, len))
        }
    }

    pub fn lane_cmp<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: &'env Self, wait: WaitList) -> Result<LaneCmpEvent<'scope>> {
        debug_assert_eq!(std::alloc::Layout::new::<i8>(), std::alloc::Layout::new::<Option<Ordering>>());

        let len = self.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorKind::InvalidBufferSize, "vectors of diferent sizes provided"))
        }

        let mut result = Buffer::<i8>::new_uninit(len, MemAccess::WRITE_ONLY, false)?;
        unsafe {
            let evt = T::vec_program().vec_cmp_partial_ord(scope, len, self, other, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            let read = change_lifetime(&result.assume_init()).read(scope, .., wait_list_from_ref(&evt))?;
            let map = read.map::<fn(Vec<i8>) -> Vec<Option<Ordering>>, Vec<Option<Ordering>>>(|x| core::mem::transmute::<Vec<i8>, Vec<Option<Ordering>>>(x));
            return Ok(map)
        }
    }

    pub fn lane_cmp_blocking (&self, other: &Self, wait: WaitList) -> Result<Vec<Option<Ordering>>> {
        debug_assert_eq!(std::alloc::Layout::new::<i8>(), std::alloc::Layout::new::<Option<Ordering>>());

        let len = self.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorKind::InvalidBufferSize, "vectors of diferent sizes provided"))
        }

        let mut result = Buffer::<i8>::new_uninit(len, MemAccess::WRITE_ONLY, false)?;
        unsafe {
            T::vec_program().vec_cmp_partial_ord_blocking(len, self, other, &mut result, [work_group_size(len)], None, wait)?;
            let v = result.assume_init().read_blocking(.., None)?;
            return Ok(transmute(v))
        }
    }

    pub fn lane_total_cmp<'scope, 'env> (&'env self, scope: &'scope Scope<'scope, 'env>, other: &'env Self, wait: WaitList) -> Result<LaneTotalCmpEvent<'scope>> {
        debug_assert_eq!(std::alloc::Layout::new::<i8>(), std::alloc::Layout::new::<Option<Ordering>>());

        let len = self.len()?;
        let mut result = Buffer::<i8>::new_uninit(len, MemAccess::WRITE_ONLY, false)?;

        unsafe {
            let evt = T::vec_program().vec_cmp_ord(scope, len, self, other, change_lifetime_mut(&mut result), [work_group_size(len)], None, wait)?;
            let read = change_lifetime(&result.assume_init()).read(scope, .., wait_list_from_ref(&evt))?;
            let map = read.map::<fn(Vec<i8>) -> Vec<Ordering>, Vec<Ordering>>(|x| core::mem::transmute::<Vec<i8>, Vec<Ordering>>(x));
            return Ok(map)
        }
    }

    pub fn lane_total_cmp_blocking (&self, other: &Self, wait: WaitList) -> Result<Vec<Ordering>> {
        debug_assert_eq!(std::alloc::Layout::new::<i8>(), std::alloc::Layout::new::<Ordering>());

        let len = self.len()?;
        if len != other.len()? {
            return Err(Error::new(ErrorKind::InvalidBufferSize, "vectors of diferent sizes provided"))
        }

        let mut result = Buffer::<i8>::new_uninit(len, MemAccess::WRITE_ONLY, false)?;
        unsafe {
            T::vec_program().vec_cmp_ord_blocking(len, self, other, &mut result, [work_group_size(len)], None, wait)?;
            let v = result.assume_init().read_blocking(.., None)?;
            return Ok(transmute(v))
        }
    }
}

impl<T: Copy> Deref for EucVec<T> {
    type Target = Buffer<T>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Debug + Copy> Debug for EucVec<T> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

impl<T: Real> PartialEq for EucVec<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        return self.eq_blocking(other, None).unwrap()
    }
}

impl<T: Real + Eq> Eq for EucVec<T> {}

macro_rules! impl_arith {
    ($($tr:ident => $fn:ident as $impl:ident),+) => {
        $(
            impl<T: Real> $tr for &EucVec<T> {
                type Output = EucVec<T>;
            
                #[inline(always)]
                fn $impl(self, rhs: Self) -> Self::Output {
                    self.$fn(rhs, None).unwrap()
                }
            }
    
            impl<T: Real> $tr<EucVec<T>> for &EucVec<T> {
                type Output = EucVec<T>;
            
                #[inline(always)]
                fn $impl(self, rhs: EucVec<T>) -> Self::Output {
                    self.$fn(&rhs, None).unwrap()
                }
            }
    
            impl<T: Real> $tr for EucVec<T> {
                type Output = EucVec<T>;
            
                #[inline(always)]
                fn $impl(self, rhs: Self) -> Self::Output {
                    self.$fn(&rhs, None).unwrap()
                }
            }
    
            impl<T: Real> $tr<&EucVec<T>> for EucVec<T> {
                type Output = EucVec<T>;
            
                #[inline(always)]
                fn $impl(self, rhs: &EucVec<T>) -> Self::Output {
                    self.$fn(rhs, None).unwrap()
                }
            }
        )+
    };
}

macro_rules! impl_scale {
    ($($tr:ident => ($fn:ident, $inv:ident) as $impl:ident),+) => {
        $(
            impl<T: Real> $tr<T> for &EucVec<T> {
                type Output = EucVec<T>;
            
                #[inline(always)]
                fn $impl(self, rhs: T) -> Self::Output {
                    self.$fn(rhs, None).unwrap()
                }
            }

            impl<T: Real> $tr<T> for EucVec<T> {
                type Output = EucVec<T>;
            
                #[inline(always)]
                fn $impl(self, rhs: T) -> Self::Output {
                    self.$fn(rhs, None).unwrap()
                }
            }

            impl_scale! { @all $tr => $inv as $impl }
        )+
    };

    (@all $tr:ident => $fn:ident as $impl:ident) => {
        impl_scale! { @(
            u8, u16, u32, u64,
            i8, i16, i32, i64,
            #[docfg(feature = "half")]
            ::half::f16,
            f32, 
            #[docfg(feature = "double")]
            f64
        ) => $tr => $fn as $impl }
    };

    (@($($(#[$meta:meta])* $t:ty),+) => $tr:ident => $fn:ident as $impl:ident) => {
        $(
            $(#[$meta])*
            impl $tr<&EucVec<$t>> for $t {
                type Output = EucVec<$t>;
            
                #[inline(always)]
                fn $impl(self, rhs: &EucVec<$t>) -> Self::Output {
                    rhs.$fn(self, None).unwrap()
                }
            }

            $(#[$meta])*
            impl $tr<EucVec<$t>> for $t {
                type Output = EucVec<$t>;
            
                #[inline(always)]
                fn $impl(self, rhs: EucVec<$t>) -> Self::Output {
                    rhs.$fn(self, None).unwrap()
                }
            }
        )+
    }
}

impl_arith! {
    Add => add_blocking as add,
    Sub => sub_blocking as sub
}

impl_scale! {
    Mul => (upscale_blocking, upscale_blocking) as mul,
    Div => (downscale_blocking, downscale_inv_blocking) as div
}

unsafe impl<T: Copy + Sync> KernelPointer<T> for EucVec<T> {
    #[inline(always)]
    unsafe fn set_arg (&self, kernel: &mut RawKernel, wait: &mut Vec<RawEvent>, idx: u32) -> Result<()> {
        self.inner.set_arg(kernel, wait, idx)
    }

    #[inline(always)]
    fn complete (&self, event: &RawEvent) -> Result<()> {
        self.inner.complete(event)
    }
}
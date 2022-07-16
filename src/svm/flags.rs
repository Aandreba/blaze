use opencl_sys::{cl_svm_mem_flags, CL_MEM_SVM_FINE_GRAIN_BUFFER, CL_MEM_SVM_ATOMICS};
use crate::buffer::flags::MemAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct SvmFlags {
    pub access: MemAccess,
    pub utils: Option<SvmUtilsFlags>
}

impl SvmFlags {
    pub const DEFAULT : Self = Self::const_new(MemAccess::READ_WRITE, None);

    #[inline(always)]
    pub fn new (access: MemAccess, utils: impl Into<Option<SvmUtilsFlags>>) -> Self {
        Self { access, utils: utils.into() }
    }

    #[inline(always)]
    pub const fn const_new (access: MemAccess, utils: Option<SvmUtilsFlags>) -> Self {
        Self { access, utils }
    }

    #[inline(always)]
    pub const fn to_bits (self) -> cl_svm_mem_flags {
        self.access.to_bits() | self.utils.map_or(0, SvmUtilsFlags::to_bits)
    }
}

impl From<MemAccess> for SvmFlags {
    #[inline(always)]
    fn from(x: MemAccess) -> Self {
        Self::new(x, None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvmUtilsFlags {
    FineGrain,
    Atomics
}

impl SvmUtilsFlags {
    #[inline(always)]
    pub const fn to_bits (self) -> cl_svm_mem_flags {
        const ATOMICS : cl_svm_mem_flags = CL_MEM_SVM_FINE_GRAIN_BUFFER | CL_MEM_SVM_ATOMICS;

        match self {
            Self::FineGrain => CL_MEM_SVM_FINE_GRAIN_BUFFER,
            Self::Atomics => ATOMICS
        }
    }

    #[inline(always)]
    pub const fn from_bits (v: cl_svm_mem_flags) -> Option<Self> {
        if v & CL_MEM_SVM_FINE_GRAIN_BUFFER == 0 {
            return None;
        }

        if v & CL_MEM_SVM_ATOMICS == 0 {
            return Some(Self::FineGrain);
        }

        Some(Self::Atomics)
    }
}

impl Into<cl_svm_mem_flags> for SvmUtilsFlags {
    #[inline(always)]
    fn into(self) -> cl_svm_mem_flags {
        self.to_bits()
    }
}

impl Into<SvmFlags> for SvmUtilsFlags {
    #[inline(always)]
    fn into(self) -> SvmFlags {
        SvmFlags::new(Default::default(), self)
    }
}
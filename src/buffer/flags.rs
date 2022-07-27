use opencl_sys::*;
use blaze_proc::docfg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub struct MemFlags {
    pub access: MemAccess,
    #[cfg_attr(docsrs, doc(cfg(feature = "cl1_2")))]
    #[cfg(feature = "cl1_2")]
    pub host_access: MemAccess,
    pub host: HostPtr
}

impl MemFlags {
    #[inline(always)]
    pub const fn new (access: MemAccess, host: HostPtr) -> Self {
        Self { access, host, #[cfg(feature = "cl1_2")] host_access: MemAccess::READ_WRITE }
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub const fn with_host_access (access: MemAccess, host_access: MemAccess, host: HostPtr) -> Self {
        Self { access, host, host_access }
    }

    #[inline(always)]
    pub const fn from_bits (bits: cl_mem_flags) -> Self {
        let access = MemAccess::from_bits(bits);
        #[cfg(feature = "cl1_2")]
        let host_access = MemAccess::from_bits_host(bits);
        let host = HostPtr::from_bits(bits);

        Self { access, #[cfg(feature = "cl1_2")] host_access, host }
    }

    #[inline(always)]
    pub const fn to_bits (self) -> cl_mem_flags {
        cfg_if::cfg_if! {
            if #[cfg(feature = "cl1_2")] {
                self.access.to_bits() | self.host.to_bits() | self.host_access.to_bits_host()
            } else {
                self.access.to_bits() | self.host.to_bits()
            }
        }
    }
}

impl Into<cl_mem_flags> for MemFlags {
    #[inline(always)]
    fn into (self) -> cl_mem_flags {
        self.to_bits()
    }
}

impl From<cl_mem_flags> for MemFlags {
    #[inline(always)]
    fn from (bits: cl_mem_flags) -> Self {
        Self::from_bits(bits)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemAccess {
    pub read: bool,
    pub write: bool
}

impl MemAccess {
    pub const NONE : Self = Self::new(false, false);
    /// This flag specifies that the memory object will be read and written by a kernel. This is the default.
    pub const READ_WRITE : Self = Self::new(true, true);
    /// This flag specifies that the memory object is a read-only memory object when used inside a kernel. Writing to a buffer or image object created with [read only](MemAccess::READ_ONLY) inside a kernel is undefined.
    pub const READ_ONLY : Self = Self::new(true, false);
    /// This flags specifies that the memory object will be written but not read by a kernel. Reading from a buffer or image object created with [write only](MemAccess::WRITE_ONLY) inside a kernel is undefined.
    pub const WRITE_ONLY : Self = Self::new(false, true);

    #[inline(always)]
    pub const fn new (read: bool, write: bool) -> Self {
        Self {
            read,
            write
        }
    }

    #[inline(always)]
    pub const fn unwrap (self) -> (bool, bool) {
        (self.read, self.write)
    }

    #[inline]
    pub const fn from_bits (flags: cl_mem_flags) -> Self {
        const READ_MASK : cl_mem_flags = CL_MEM_READ_WRITE | CL_MEM_READ_ONLY;
        const WRITE_MASK : cl_mem_flags = CL_MEM_READ_WRITE | CL_MEM_WRITE_ONLY;

        let read = (flags & READ_MASK) != 0;
        let write = (flags & WRITE_MASK) != 0;

        Self::new(read, write)
    }

    #[docfg(feature = "cl1_2")]
    #[inline]
    pub const fn from_bits_host (flags: cl_mem_flags) -> Self {
        if flags & CL_MEM_HOST_NO_ACCESS != 0 {
            return Self::NONE;
        }

        let read = (flags & CL_MEM_HOST_WRITE_ONLY) == 0;
        let write = (flags & CL_MEM_HOST_READ_ONLY) == 0;
        Self::new(read, write)
    }

    #[inline(always)]
    pub const fn to_bits (self) -> cl_mem_flags {
        match self.unwrap() {
            (true, true) => CL_MEM_READ_WRITE,
            (true, false) => CL_MEM_READ_ONLY,
            (false, true) => CL_MEM_WRITE_ONLY,
            (false, false) => 0
        }
    }

    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub const fn to_bits_host (self) -> cl_mem_flags {
        match self.unwrap() {
            (true, true) => 0,
            (true, false) => CL_MEM_HOST_READ_ONLY,
            (false, true) => CL_MEM_HOST_WRITE_ONLY,
            (false, false) => CL_MEM_HOST_NO_ACCESS
        }
    }
}

impl Default for MemAccess {
    #[inline(always)]
    fn default() -> Self {
        Self::READ_WRITE
    }
}

impl Into<MemFlags> for MemAccess {
    #[inline(always)]
    fn into(self) -> MemFlags {
        MemFlags::new(self, HostPtr::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostPtr {
    Use,
    Other (bool, bool)
}

impl HostPtr {
    pub const NONE : Self = Self::new(false, false);
    /// This flag is valid only if host_ptr is not NULL. If specified, it indicates that the application wants the OpenCL implementation to use memory referenced by host_ptr as the storage bits for the memory object.
    pub const USE : Self = Self::Use;
    /// This flag specifies that the application wants the OpenCL implementation to allocate memory from host accessible memory.
    pub const ALLOC : Self = Self::new(true, false);
    /// This flag is valid only if host_ptr is not NULL. If specified, it indicates that the application wants the OpenCL implementation to allocate memory for the memory object and copy the data from memory referenced by host_ptr.
    pub const COPY : Self = Self::new(false, true);
    /// ```ALLOC``` and ```COPY``` combined
    pub const ALLOC_COPY : Self = Self::new(true, true);

    #[inline(always)]
    pub const fn new (alloc: bool, copy: bool) -> Self {
        Self::Other(alloc, copy)
    }

    #[inline(always)]
    pub const fn is_use (&self) -> bool {
        match self {
            Self::Use => true,
            _ => false
        }
    }

    #[inline(always)]
    pub const fn is_alloc (&self) -> bool {
        match self {
            Self::Other (true, _) => true,
            _ => false
        }
    }

    #[inline(always)]
    pub const fn is_copy (&self) -> bool {
        match self {
            Self::Other (_, true) => true,
            _ => false
        }
    }

    #[inline(always)]
    pub const fn from_bits (flags: cl_mem_flags) -> Self {
        if flags & CL_MEM_USE_HOST_PTR != 0 { return Self::USE; }

        let alloc = flags & CL_MEM_ALLOC_HOST_PTR != 0;
        let copy = flags & CL_MEM_COPY_HOST_PTR != 0;

        Self::new(alloc, copy)
    }

    #[inline(always)]
    pub const fn to_bits (self) -> cl_mem_flags {
        match self {
            Self::Use => CL_MEM_USE_HOST_PTR,
            Self::Other (alloc, copy) => {
                let mut bits = 0;
                if alloc { bits |= CL_MEM_ALLOC_HOST_PTR }
                if copy { bits |= CL_MEM_COPY_HOST_PTR }
                bits
            }
        }
    }
}

impl Default for HostPtr {
    #[inline(always)]
    fn default() -> Self {
        Self::NONE
    }
}

impl Into<MemFlags> for HostPtr {
    #[inline(always)]
    fn into(self) -> MemFlags {
        MemFlags::new(MemAccess::default(), self)
    }
}
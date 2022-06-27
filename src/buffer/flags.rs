use opencl_sys::{cl_mem_flags, CL_MEM_READ_WRITE, CL_MEM_WRITE_ONLY, CL_MEM_READ_ONLY, CL_MEM_USE_HOST_PTR, CL_MEM_ALLOC_HOST_PTR, CL_MEM_COPY_HOST_PTR};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct MemFlags {
    pub access: MemAccess,
    pub alloc: bool
}

impl MemFlags {
    #[inline(always)]
    pub const fn new (access: MemAccess, alloc_host: bool) -> Self {
        Self { access, alloc: alloc_host }
    }

    #[inline(always)]
    pub const fn from_full (flags: FullMemFlags) -> Self {
        FullMemFlags::to_reduced(flags)
    }

    #[inline(always)]
    pub const fn to_full (self) -> FullMemFlags {
        FullMemFlags::from_reduced(self)
    }

    #[inline(always)]
    pub const fn from_bits (bits: cl_mem_flags) -> Self {
        let access = MemAccess::from_bits(bits);
        let alloc = bits & CL_MEM_ALLOC_HOST_PTR != 0;
        Self::new(access, alloc)
    }

    #[inline(always)]
    pub const fn to_bits (self) -> cl_mem_flags {
        self.access.to_bits() | match self.alloc {
            true => CL_MEM_ALLOC_HOST_PTR,
            _ => 0
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FullMemFlags {
    pub access: MemAccess,
    pub host: HostPtr
}

impl FullMemFlags {
    #[inline(always)]
    pub const fn new (access: MemAccess, host: HostPtr) -> Self {
        Self { access, host }
    }

    #[inline(always)]
    pub const fn from_bits (bits: cl_mem_flags) -> Self {
        let access = MemAccess::from_bits(bits);
        let host = HostPtr::from_bits(bits);
        Self::new(access, host)
    }

    #[inline(always)]
    pub const fn to_bits (self) -> cl_mem_flags {
        self.access.to_bits() | self.host.to_bits()
    }

    #[inline(always)]
    pub const fn from_reduced (v: MemFlags) -> Self {
        let host = HostPtr::new(v.alloc, false);
        Self::new(v.access, host)
    }

    #[inline(always)]
    pub const fn to_reduced (self) -> MemFlags {
        MemFlags::new(self.access, self.host.is_alloc())
    }
}

impl Into<cl_mem_flags> for FullMemFlags {
    #[inline(always)]
    fn into (self) -> cl_mem_flags {
        self.to_bits()
    }
}

impl From<cl_mem_flags> for FullMemFlags {
    #[inline(always)]
    fn from (bits: cl_mem_flags) -> Self {
        Self::from_bits(bits)
    }
}

impl Into<MemFlags> for FullMemFlags {
    #[inline(always)]
    fn into(self) -> MemFlags {
        self.to_reduced()
    }
}

impl From<MemFlags> for FullMemFlags {
    #[inline(always)]
    fn from(v: MemFlags) -> Self {
        v.to_full()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemAccess {
    pub read: bool,
    pub write: bool
}

impl MemAccess {
    /// This flag specifies that the memory object will be read and written by a kernel. This is the default.
    pub const READ_WRITE : Self = Self::new(true, true);
    /// This flag specifies that the memory object is a read-only memory object when used inside a kernel. Writing to a buffer or image object created with CL_MEM_READ_ONLY inside a kernel is undefined.
    pub const READ_ONLY : Self = Self::new(true, false);
    /// This flags specifies that the memory object will be written but not read by a kernel. Reading from a buffer or image object created with CL_MEM_WRITE_ONLY inside a kernel is undefined.
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

    #[inline(always)]
    pub const fn to_bits (self) -> cl_mem_flags {
        match self.unwrap() {
            (true, true) => CL_MEM_READ_WRITE,
            (true, false) => CL_MEM_READ_ONLY,
            (false, true) => CL_MEM_WRITE_ONLY,
            (false, false) => 0
        }

    }
}

impl Default for MemAccess {
    #[inline(always)]
    fn default() -> Self {
        Self::READ_WRITE
    }
}

impl From<cl_mem_flags> for MemAccess {
    #[inline(always)]
    fn from(x: cl_mem_flags) -> Self {
        Self::from_bits(x)
    }
}

impl Into<cl_mem_flags> for MemAccess {
    #[inline(always)]
    fn into(self) -> cl_mem_flags {
        self.to_bits()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostPtr {
    Use,
    Other (bool, bool)
}

impl HostPtr {
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
        Self::Other(false, false)
    }
}

impl From<cl_mem_flags> for HostPtr {
    #[inline(always)]
    fn from(v: cl_mem_flags) -> Self {
        Self::from_bits(v)
    }
}

impl Into<cl_mem_flags> for HostPtr {
    #[inline(always)]
    fn into(self) -> cl_mem_flags {
        self.to_bits()
    }
}
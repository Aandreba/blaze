use super::*;
use std::{mem::MaybeUninit};
use opencl_sys::{cl_command_queue, CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE, CL_QUEUE_PROPERTIES, clRetainCommandQueue, clReleaseCommandQueue, clFlush, clFinish, cl_command_queue_info, clGetCommandQueueInfo, cl_context, CL_QUEUE_CONTEXT, CL_QUEUE_DEVICE, CL_QUEUE_REFERENCE_COUNT, cl_command_queue_properties, CL_QUEUE_PROFILING_ENABLE};
use rscl_proc::docfg;
use crate::context::RawContext;
use std::ptr::addr_of_mut;

#[repr(transparent)]
pub struct CommandQueue (cl_command_queue);

impl CommandQueue {
    #[docfg(not(feature = "cl2"))]
    pub fn new (props: CommandQueueProperties, ctx: &RawContext, device: &Device) -> Result<Self> {
        let props = props.to_bits();
        
        let mut err = 0;
        let id = unsafe {
            opencl_sys::clCreateCommandQueue(ctx.id(), device.id(), props, addr_of_mut!(err))
        };

        if err != 0 {
            return Err(Error::from(err));
        }

        Ok(Self(id))
    }

    #[docfg(feature = "cl2")]
    pub fn new (props: impl Into<QueueProperties>, ctx: &RawContext, device: &Device) -> Result<Self> {
        use elor::prelude::*;

        let props = props.into().to_bits();
        let props = match props {
            Left(x) => x.as_ptr(),
            Right(x) => x.as_ptr()
        };
        
        let mut err = 0;
        let id = unsafe {
            opencl_sys::clCreateCommandQueueWithProperties(ctx.id(), device.id(), props, addr_of_mut!(err))
        };

        if err != 0 {
            return Err(Error::from(err));
        }

        Ok(Self(id))
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_command_queue {
        self.0
    }

    /// Return the context specified when the command-queue is created.
    #[inline(always)]
    pub fn context (&self) -> Result<cl_context> {
        self.get_info(CL_QUEUE_CONTEXT)
    }

    /// Return the device specified when the command-queue is created.
    #[inline(always)]
    pub fn device (&self) -> Result<Device> {
        self.get_info(CL_QUEUE_DEVICE)
    }
    
    /// Return the command-queue reference count.
    #[inline(always)]
    pub fn reference_count (&self) -> Result<u32> {
        self.get_info(CL_QUEUE_REFERENCE_COUNT)
    }

    /// Return the currently specified properties for the command-queue.
    #[inline(always)]
    pub fn properties (&self) -> Result<CommandQueueProperties> {
        let props = self.get_info(CL_QUEUE_PROPERTIES)?;
        Ok(CommandQueueProperties::from_bits(props))
    }

    /// Return the size of the device command-queue. To be considered valid for this query, command_queue must be a device command-queue.
    #[docfg(feature = "cl2")]
    #[inline(always)]
    pub fn size (&self) -> Result<u32> {
        self.get_info(opencl_sys::CL_QUEUE_SIZE)
    }

    /// Return the current default command queue for the underlying device.
    #[docfg(feature = "cl2_1")]
    #[inline(always)]
    pub fn device_default (&self) -> Result<CommandQueue> {
        self.get_info(opencl_sys::CL_QUEUE_DEVICE_DEFAULT)
    }

    #[inline(always)]
    pub fn flush (&self) -> Result<()> {
        unsafe {
            tri!(clFlush(self.0));
        }

        Ok(())
    }

    #[inline(always)]
    pub fn finish (&self) -> Result<()> {
        unsafe {
            tri!(clFinish(self.0));
        }

        Ok(())
    }

    #[inline]
    fn get_info<T> (&self, ty: cl_command_queue_info) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();
        unsafe {
            tri!(clGetCommandQueueInfo(self.0, ty, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

impl Clone for CommandQueue {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainCommandQueue(self.0))
        }

        Self(self.0)
    }
}

impl Drop for CommandQueue {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseCommandQueue(self.0))
        }
    }
}

unsafe impl Send for CommandQueue {}
unsafe impl Sync for CommandQueue {}

cfg_if::cfg_if! {
    if #[cfg(feature = "cl2")] {
        use core::num::NonZeroU32;
        use opencl_sys::{cl_queue_properties, CL_QUEUE_SIZE};
        use elor::prelude::*;

        #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[non_exhaustive]
        pub struct QueueProperties {
            pub props: CommandQueueProperties,
            /// Specifies the size of the device queue in bytes.\
            /// This can only be specified if [on device](OutOfOrderExec::OnDevice) is set in `props`. This must be a value less or equal to the [max size](Device::queue_max_size).\
            /// For best performance, this should be less or equal to the [preferred size](Device::queue_preferred_size).\
            /// If `size` is not specified, the device queue is created with the [preferred size](Device::queue_preferred_size) as the size of the queue.
            pub size: Option<NonZeroU32>
        }

        impl QueueProperties {
            #[inline(always)]
            pub fn new (props: CommandQueueProperties, size: impl Into<Option<NonZeroU32>>) -> Self {
                Self { 
                    props,
                    size: size.into()
                }
            }

            #[inline(always)]
            pub const fn const_new (props: CommandQueueProperties, size: Option<NonZeroU32>) -> Self {
                Self { props, size }
            }

            #[inline(always)]
            pub fn to_bits (self) -> Either<[cl_queue_properties; 5], [cl_queue_properties; 3]> {
                let props = self.props.to_bits();

                if let Some(size) = self.size {
                    return Left (
                        [
                            CL_QUEUE_PROPERTIES as cl_queue_properties, props,
                            CL_QUEUE_SIZE as cl_queue_properties, size.get() as cl_queue_properties,
                            0
                        ]
                    )
                }

                Right([CL_QUEUE_PROPERTIES as cl_queue_properties, props, 0])
            }
        }

        impl From<CommandQueueProperties> for QueueProperties {
            #[inline(always)]
            fn from (props: CommandQueueProperties) -> Self {
                Self::new(props, None)
            }
        }
    } else {
        pub type QueueProperties = CommandQueueProperties;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub struct CommandQueueProperties {
    /// Determines whether the commands queued in the command-queue are executed in-order or out-of-order.
    pub out_of_order_exec: OutOfOrderExec,
    /// Enable or disable profiling of commands in the command-queue. If set, the profiling of commands is enabled. Otherwise profiling of commands is disabled.
    pub profiling: bool
}

impl CommandQueueProperties {
    #[inline(always)]
    pub fn new (out_of_order_exec: impl Into<OutOfOrderExec>, profiling: bool) -> Self {
        Self {
            out_of_order_exec: out_of_order_exec.into(),
            profiling
        }
    }

    #[inline(always)]
    pub const fn const_new (out_of_order_exec: OutOfOrderExec, profiling: bool) -> Self {
        Self {
            out_of_order_exec,
            profiling
        }
    }

    #[inline(always)]
    pub const fn from_bits (v: cl_command_queue_properties) -> Self {
        let out_of_order_exec = OutOfOrderExec::from_bits(v);
        let profiling = v & CL_QUEUE_PROFILING_ENABLE != 0;
        Self { out_of_order_exec, profiling }
    }

    #[inline(always)]
    pub const fn to_bits (self) -> cl_command_queue_properties {
        let mut bits = self.out_of_order_exec.to_bits();
        if self.profiling {
            bits |= CL_QUEUE_PROFILING_ENABLE
        }

        bits
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OutOfOrderExec {
    /// The commands in the command-queue are executed out-of-order
    Enabled,
    /// Commands are executed in-order
    Disabled,
    /// Indicates that this is a device queue, and the commands in the queue are executed out-of-order.\
    /// The boolean value indicates if this is the default device queue or not.
    #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
    #[cfg(feature = "cl2")]
    OnDevice (bool)
}

impl OutOfOrderExec {
    #[inline]
    pub const fn from_bits (v: cl_command_queue_properties) -> Self {
        if v & CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE == 0 {
            return Self::Disabled;
        }

        #[cfg(feature = "cl2")]
        if v & opencl_sys::CL_QUEUE_ON_DEVICE == 0 {
            return Self::Enabled;
        }

        #[cfg(feature = "cl2")]
        if v & opencl_sys::CL_QUEUE_ON_DEVICE_DEFAULT == 0 {
            return Self::OnDevice (false);
        }

        #[cfg(feature = "cl2")]
        return Self::OnDevice (true);
        #[cfg(not(feature = "cl2"))]
        return Self::Enabled;
    }

    #[inline]
    pub const fn to_bits (self) -> cl_command_queue_properties {
        #[cfg(feature = "cl2")]
        const ON_DEVICE : cl_command_queue_properties = opencl_sys::CL_QUEUE_ON_DEVICE | CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE;
        #[cfg(feature = "cl2")]
        const ON_DEVICE_DEFAULT : cl_command_queue_properties = opencl_sys::CL_QUEUE_ON_DEVICE_DEFAULT | ON_DEVICE;

        match self {
            Self::Enabled => CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE,
            #[cfg(feature = "cl2")]
            Self::OnDevice (false) => ON_DEVICE,
            #[cfg(feature = "cl2")]
            Self::OnDevice (true) => ON_DEVICE_DEFAULT,
            Self::Disabled => 0
        }
    }
}

impl From<bool> for OutOfOrderExec {
    #[inline(always)]
    fn from(v: bool) -> Self {
        if v.into() {
            return Self::Enabled
        }

        Self::Disabled
    }
}

impl Default for OutOfOrderExec {
    #[inline(always)]
    fn default() -> Self {
        Self::Disabled
    }
}
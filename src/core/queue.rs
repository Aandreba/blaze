use super::*;
use std::{mem::MaybeUninit, ptr::NonNull, ffi::c_void};
use opencl_sys::*;
use blaze_proc::docfg;
use crate::{context::RawContext, prelude::RawEvent, wait_list, WaitList};
use std::ptr::addr_of_mut;

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RawCommandQueue (NonNull<c_void>);

impl RawCommandQueue {
    #[cfg(not(feature = "cl2"))]
    pub fn new (ctx: &RawContext, props: CommandQueueProperties, device: &RawDevice) -> Result<Self> {
        let props = props.to_bits();
        let mut err = 0;

        let id = unsafe {
            opencl_sys::clCreateCommandQueue(ctx.id(), device.id(), props, addr_of_mut!(err))
        };

        if err != 0 {
            return Err(Error::from(err));
        }

        Ok(NonNull::new(id).map(Self).unwrap())
    }

    #[cfg(feature = "cl2")]
    pub fn new (ctx: &RawContext, props: impl Into<QueueProperties>, device: &RawDevice) -> Result<Self> {
        let props : QueueProperties = props.into();
        let mut err = 0;
        let id;

        cfg_if::cfg_if! {
            if #[cfg(feature = "strict")] {
                let props = match props.to_bits() {
                    Left(x) => x.as_ptr(),
                    Right(x) => x.as_ptr()
                };
                
                id = unsafe { opencl_sys::clCreateCommandQueueWithProperties(ctx.id(), device.id(), props, addr_of_mut!(err)) };
            } else {
                #[allow(deprecated)]
                if ctx.greatest_common_version()? < device::Version::CL2 {
                    id = unsafe { opencl_sys::clCreateCommandQueue(ctx.id(), device.id(), props.props.to_bits(), addr_of_mut!(err)) }
                } else {
                    let props = match props.to_bits() {
                        Left(x) => x.as_ptr(),
                        Right(x) => x.as_ptr()
                    };
                    
                    id = unsafe { opencl_sys::clCreateCommandQueueWithProperties(ctx.id(), device.id(), props, addr_of_mut!(err)) };
                }
            }
        }

        if err != 0 {
            return Err(Error::from(err));
        }

        Ok(NonNull::new(id).map(Self).unwrap())
    }

    #[inline(always)]
    pub const unsafe fn from_id (id: cl_command_queue) -> Option<Self> {
        NonNull::new(id).map(Self)
    }

    #[inline(always)]
    pub const unsafe fn from_id_unchecked (id: cl_command_queue) -> Self {
        Self(NonNull::new_unchecked(id))
    }

    #[inline(always)]
    pub const fn id (&self) -> cl_command_queue {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub unsafe fn retain (&self) -> Result<()> {
        tri!(clRetainCommandQueue(self.id()));
        Ok(())
    }

    /// Return the context specified when the command-queue is created.
    #[inline(always)]
    pub fn context (&self) -> Result<RawContext> {
        let ctx = self.get_info::<cl_context>(CL_QUEUE_CONTEXT)?;
        unsafe { 
            tri!(clRetainContext(ctx));
            // SAFETY: Context checked to be valid by `clRetainContext`.
            Ok(RawContext::from_id_unchecked(ctx))
        }
    }

    /// Return the device specified when the command-queue is created.
    #[inline(always)]
    pub fn device (&self) -> Result<RawDevice> {
        let dev = self.get_info::<cl_device_id>(CL_QUEUE_DEVICE)?;
        unsafe {
            cfg_if::cfg_if! {
                if #[cfg(feature = "cl1_2")] {
                    tri!(clRetainDevice(dev));
                    // SAFETY: Context checked to be valid by `clRetainContext`.
                    Ok(RawDevice::from_id_unchecked(dev))
                } else {
                    if let Some(dev) = RawDevice::from_id(dev) {
                        return Ok(dev);
                    }

                    Err(ErrorType::InvalidDevice.into())
                }
            }
        }
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

    /// Return the properties argument specified in creation.
    #[docfg(feature = "cl3")]
    #[inline(always)]
    pub fn queue_properties (&self) -> Result<QueueProperties> {
        let v = self.get_info_array::<cl_queue_properties>(opencl_sys::CL_QUEUE_PROPERTIES_ARRAY)?;
        Ok(QueueProperties::from_bits(&v))
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
    pub fn device_default (&self) -> Result<RawCommandQueue> {
        // TODO FIX
        let queue = self.get_info::<cl_command_queue>(opencl_sys::CL_QUEUE_DEVICE_DEFAULT)?;
        
        unsafe {
            tri!(clRetainCommandQueue(queue));
            // SAFETY: Queue checked to be valid by `clRetainCommandQueue`.
            Ok(RawCommandQueue::from_id_unchecked(queue))
        }
    }

    /// Issues all previously queued OpenCL commands in a command-queue to the device associated with the command-queue.
    #[inline(always)]
    pub fn flush (&self) -> Result<()> {
        unsafe {
            tri!(clFlush(self.id()));
        }

        Ok(())
    }

    /// Blocks the current thread until all previously queued OpenCL commands in a command-queue are issued to the associated device and have completed.
    #[inline(always)]
    pub fn finish (&self) -> Result<()> {
        unsafe {
            tri!(clFinish(self.id()));
        }

        Ok(())
    }

    /// A synchronization point that enqueues a barrier operation.\
    /// If the wait list is empty, then this particular command waits until all previous enqueued commands to command_queue have completed.
    /// The barrier command either waits for a list of events to complete, or if the list is empty it waits for all commands previously enqueued in the queue to complete before it completes.
    /// This command blocks command execution, that is, any following commands enqueued after it do not execute until it completes. 
    /// This command returns an event which can be waited on, i.e. this event can be waited on to insure that all events either in the wait list or all previously enqueued commands, 
    /// queued before this command to the command queue, have completed.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn barrier (&self, wait: WaitList) -> Result<crate::prelude::RawEvent> {
        let (num_events_in_wait_list, event_wait_list) = wait_list(wait);

        let mut evt = core::ptr::null_mut();
        unsafe {
            tri!(clEnqueueBarrierWithWaitList(self.id(), num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
            Ok(crate::prelude::RawEvent::from_id(evt).unwrap())
        }
    }

    /// Enqueues a marker command which waits for either a list of events to complete, or all previously enqueued commands to complete.
    #[docfg(feature = "cl1_2")]
    #[inline(always)]
    pub fn marker (&self, wait: WaitList) -> Result<crate::prelude::RawEvent> {
        let (num_events_in_wait_list, event_wait_list) = wait_list(wait);

        let mut evt = core::ptr::null_mut();
        unsafe {
            tri!(clEnqueueMarkerWithWaitList(self.id(), num_events_in_wait_list, event_wait_list, addr_of_mut!(evt)));
            Ok(crate::prelude::RawEvent::from_id(evt).unwrap())
        }
    }

    #[inline]
    fn get_info<T: Copy> (&self, ty: cl_command_queue_info) -> Result<T> {
        let mut result = MaybeUninit::<T>::uninit();
        unsafe {
            tri!(clGetCommandQueueInfo(self.id(), ty, core::mem::size_of::<T>(), result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }

    #[allow(unused)]
    #[inline]
    fn get_info_array<T: Copy> (&self, ty: cl_command_queue_info) -> Result<Box<[T]>> {
        let mut size = 0;
        unsafe {
            tri!(clGetCommandQueueInfo(self.id(), ty, 0, core::ptr::null_mut(), addr_of_mut!(size)));
        }

        let mut result = Box::new_uninit_slice(size / core::mem::size_of::<T>());
        unsafe {
            tri!(clGetCommandQueueInfo(self.id(), ty, size, result.as_mut_ptr().cast(), core::ptr::null_mut()));
            Ok(result.assume_init())
        }
    }
}

impl Clone for RawCommandQueue {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            tri_panic!(clRetainCommandQueue(self.id()))
        }

        Self(self.0)
    }
}

impl Drop for RawCommandQueue {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            tri_panic!(clReleaseCommandQueue(self.id()))
        }
    }
}

unsafe impl Send for RawCommandQueue {}
unsafe impl Sync for RawCommandQueue {}

cfg_if::cfg_if! {
    if #[cfg(feature = "cl2")] {
        use core::num::NonZeroU32;
        use opencl_sys::{cl_queue_properties, CL_QUEUE_SIZE};
        use elor::prelude::*;

        #[cfg_attr(docsrs, doc(cfg(feature = "cl2")))]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[non_exhaustive]
        pub struct QueueProperties {
            pub props: CommandQueueProperties,
            /// Specifies the size of the device queue in bytes.\
            /// This can only be specified if [on device](OutOfOrderExec::OnDevice) is set in `props`. This must be a value less or equal to the [max size](crate::prelude::RawDevice::queue_on_device_max_size).\
            /// For best performance, this should be less or equal to the [preferred size](crate::prelude::RawDevice::queue_on_device_preferred_size).\
            /// If `size` is not specified, the device queue is created with the [preferred size](crate::prelude::RawDevice::queue_on_device_preferred_size) as the size of the queue.
            pub size: Option<NonZeroU32>
        }

        impl QueueProperties {
            const PROPERTIES : cl_queue_properties = CL_QUEUE_PROPERTIES as cl_queue_properties;
            const SIZE : cl_queue_properties = CL_QUEUE_SIZE as cl_queue_properties;

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
                            Self::PROPERTIES, props,
                            Self::SIZE, size.get() as cl_queue_properties,
                            0
                        ]
                    )
                }

                Right([CL_QUEUE_PROPERTIES as cl_queue_properties, props, 0])
            }

            #[inline]
            pub fn from_bits (bits: &[cl_queue_properties]) -> Self {
                if bits.len() == 0 {
                    return Self::default()
                }

                let mut props = CommandQueueProperties::default();
                let mut size = None;

                match bits[0] {
                    Self::PROPERTIES => props = CommandQueueProperties::from_bits(bits[1]),
                    Self::SIZE => size = NonZeroU32::new(u32::try_from(bits[1]).unwrap()),
                    0 => return Self::new(props, size),
                    _ => panic!()
                }

                match bits[2] {
                    Self::PROPERTIES => props = CommandQueueProperties::from_bits(bits[3]),
                    Self::SIZE => size = NonZeroU32::new(u32::try_from(bits[3]).unwrap()),
                    0 => return Self::new(props, size),
                    _ => panic!()
                }

                Self::new(props, size)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl Default for CommandQueueProperties {
    fn default() -> Self {
        Self { 
            out_of_order_exec: Default::default(), 
            #[cfg(any(test, debug_assertions))]
            profiling: true,
            #[cfg(not(any(test, debug_assertions)))]
            profiling: false
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        if v {
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
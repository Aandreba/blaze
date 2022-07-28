flat_mod!(error, platform, program, queue, kernel);

pub mod device;
pub use device::RawDevice;

#[cfg(feature = "cl2")]
flat_mod!(pipe);
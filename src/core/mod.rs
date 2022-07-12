flat_mod!(error, platform, program, queue, kernel);

pub mod device;
pub use device::Device;

#[cfg(feature = "cl2")]
flat_mod!(pipe);
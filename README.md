# Features
## Image ##
Enables OpenCL image support, with help of the [rust-ffmpef](https://github.com/zmwangx/rust-ffmpeg) crate

## Strict ##
When the `strict` feature is enabled, `blaze` will not check for OpenCL support for the specified version at runtime, increasing perfomance. 
When disabled, `blaze` will dynamically check the OpenCL version at runtime (when needed), and make adjustments to ensure the maximum compatiblity possible.
This feature is disabled by default.
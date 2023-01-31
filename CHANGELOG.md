0.1.0
- Complete overhaul of the event system
- Removed need for double allocations to store callbacks (gone from `Box<Box<dyn Fn(...)>>` to `ThinBox<dyn Fn(...)>`)
    - Custom implementation of `ThinBox`, based on the one from the standard library
- Added buffer slices
- Improved OpenCL shared library finding (specifically, for NVIDIA on Windows)
- Etc.
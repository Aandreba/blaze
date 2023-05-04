pub fn main() {
    #[cfg(windows)]
    include_opencl();
}

#[cfg(windows)]
fn include_opencl() {
    if let Some(path) = option_env!("CUDA_PATH") {
        let lib = Utf8Path::new(path).join("lib");
        #[cfg(target_pointer_width = "32")]
        let path = lib.join("Win32");
        #[cfg(target_pointer_width = "64")]
        let path = lib.join("x64");
        println!("cargo:rustc-link-search={path}");
    } else {
        eprintln!("OpenCL library path not found. This may result in an error in Windows systems.")
    }
}

use rscl_proc::rscl;

rscl! {
    pub struct Arith {
        kernel void add (const ulong n, __global const float* inn, __global const float* rhs, __global float* out) {
            for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
                out[id] = inn[id] + rhs[id];
            }
        }
    }
}
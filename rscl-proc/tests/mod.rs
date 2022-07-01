use rscl_proc::rscl;

rscl! {
    kernel void add (const ulong n, __global const float* rhs, __global const float* inn, __global float* out) {
        for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
            out[id] = inn[id] + rhs[id];
        }
    }
}
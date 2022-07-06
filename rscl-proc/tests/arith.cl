kernel void sum (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
    for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
        int two = (int)in[id];
        out[id] = in[id] + rhs[id];
    }
}
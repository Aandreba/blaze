#ifndef PRECISION
    typedef float real;
#endif

// vector - vector arithmetic 
kernel void add (const uint n, global const real* lhs, global const real* rhs, global real* res) {
    for (uint i = get_global_id(0); i < n; i += get_global_size(0)) {
        res[i] = lhs[i] + rhs[i];
    }
}

kernel void sub (const uint n, global const real* lhs, global const real* rhs, global real* res) {
    for (uint i = get_global_id(0); i < n; i += get_global_size(0)) {
        res[i] = lhs[i] - rhs[i];
    }
}
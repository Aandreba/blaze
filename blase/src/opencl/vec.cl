#ifndef PRECISION
    typedef float real;
    typedef uint usize;
#endif

// vector - vector arithmetic 
kernel void add (const usize n, global const real* lhs, global const real* rhs, global real* res) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        res[i] = lhs[i] + rhs[i];
    }
}

kernel void sub (const usize n, global const real* lhs, global const real* rhs, global real* res) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        res[i] = lhs[i] - rhs[i];
    }
}

kernel void scal (const usize n, const real alpha, global const real* x, global real* res) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        res[i] = alpha * x[i];
    }
}
#ifndef PRECISION
    typedef float real;
    typedef uint usize;
#endif

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

kernel void scal_down (const usize n, global const real* x, const real alpha, global real* res) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        res[i] = x[i] / alpha;
    }
}

kernel void scal_down_inv (const usize n, const real alpha, global const real* x, global real* res) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        res[i] = alpha / x[i];
    }
}

kernel void cmp (const usize n, const global real* lhs, const global real* rhs, global volatile uint* res) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        uint j = i / 32;
        uint k = i % 32;
        
        if (lhs[i] == rhs[i]) {
            atomic_or(&res[j], 1 << k);
        } else {
            atomic_and(&res[j], ~(1 << k));
        }
    }
}
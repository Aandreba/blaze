#ifndef PRECISION
    #define WGS1 1024
    #define WGS2 1024
    #define PRECISION 32
    #define ISFLOAT false
    #define ISSIGNED false
    typedef float real;
    typedef uint usize;
#endif

global uint mutex = 0;

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

kernel void sum (const usize n, global const real* x, global real* res) {
    real local_sum = 0;
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        local_sum += x[i];
    }

    while (atomic_cmpxchg(&mutex, 0, 1) != 0) {}
    *res += local_sum;
    atomic_xchg(&mutex, 0);
}

kernel void sum_cpu (const usize n, global const real* x, global real* res) {
    real local_sum = 0;
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        local_sum += x[i];
    }

    res[get_global_id(0)] = local_sum;
}

kernel void sum_atomic (const usize n, global const uint* x, global uint* res) {
    uint local_sum = 0;
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        local_sum += x[i];
    }

    atomic_add(res, local_sum);
}
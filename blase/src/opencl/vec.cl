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
    local uint mutex;
    if (get_global_id(0) == 0) mutex = 0;
    barrier(CLK_LOCAL_MEM_FENCE);

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
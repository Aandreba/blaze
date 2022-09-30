#ifndef PRECISION
    #define ISFLOAT true
    #define PRECISION 32
    #define ORD_NONE 2
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

kernel void eq (const usize n, const global real* lhs, const global real* rhs, global volatile uint* res) {
    if (get_local_id(0) == 0)
       *res = 1;
    work_group_barrier(CLK_LOCAL_MEM_FENCE);

    if (atomic_load(&res) == 1) {
        for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {            
            if (lhs[i] != rhs[i]) {
                atomic_store(&res, 0);
                break;
            }
        }
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

kernel void ord (const usize n, const global real* lhs, const global real* rhs, global char* res) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {

        // Float total_cmp from rust
        // https://doc.rust-lang.org/stable/std/primitive.f32.html#method.total_cmp
        #if ISFLOAT
            #if PRECISION == 16
                typedef short bits;
                typedef ushort ubits;
            #elif PRECISION == 32
                typedef int bits;
                typedef uint ubits;
            #elif PRECISION == 64
                typedef long bits;
                typedef ulong ubits;
            #endif

            union { real a; bits b; } w;

            // Get left bits
            w.a = lhs[i];
            bits left = w.b;

            // Get right bits
            w.a = rhs[i];
            bits right = w.b;

            left ^= (bits)((ubits)(left >> (PRECISION - 1)) >> 1);
            right ^= (bits)((ubits)(right >> (PRECISION - 1)) >> 1);
        #else
            const real left = lhs[i];
            const real right = rhs[i];
        #endif

        if (left == right) {
            res[i] = 0;
        } else if (left < right) {
            res[i] = -1;
        } else {
            res[i] = 1;
        }
    }
}

kernel void partial_ord (const usize n, const global real* lhs, const global real* rhs, global char* res) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        const real left = lhs[i];
        const real right = rhs[i];

        if (left < right) {
            res[i] = -1;
        } else if (left > right) {
            res[i] = 1;
        }

        #if ISFLOAT
            else if (left != right) {
                res[i] = ORD_NONE;
            }
        #endif

        else {
            res[i] = 0;
        }
    }
}
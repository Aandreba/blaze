//typedef uint usize;
#define MULTIPLIER 0x5DEECE66Dl
#define ADDEND 0xBl
#define MASK() (1l << 48) - 1;
#define FLOAT_UNIT() ((float)(1 << 24));

static inline uint next (const usize bits, global ulong* seed) {
    ulong next = (*seed * MULTIPLIER + ADDEND) & MASK();
    *seed = next;
    return (uint)(next >> (48 - bits));
}

kernel void random_uchar (const usize n, global ulong* seed, global uchar* out, const uchar origin, const uchar delta) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        uchar v = (uchar)next(8, &seed[get_global_id(0)]);
        out[i] = (v % delta) + origin;
    }
}

kernel void random_ushort (const usize n, global ulong* seed, global ushort* out, const ushort origin, const ushort delta) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) { 
        uchar v = (ushort)next(16, &seed[get_global_id(0)]);
        out[i] = (v % delta) + origin;
    }
}

kernel void random_uint (const usize n, global ulong* seed, global uint* out, const uint origin, const uint delta) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) { 
        uchar v = next(16, &seed[get_global_id(0)]);
        out[i] = (v % delta) + origin;
    }
}

kernel void random_ulong (const usize n, global ulong* seed, global ulong* out, const ulong origin, const ulong delta) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        ulong v = ((ulong)next(32, &seed[get_global_id(0)]) << 32) | (ulong)next(32, &seed[get_global_id(0)]);
        out[i] = (v % delta) + origin;
    }
}

kernel void random_char (const usize n, global ulong* seed, global char* out, const char origin, const char delta) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) { 
        char v = (char)next(7, &seed[get_global_id(0)]);
        out[i] = (v % delta) + origin;
    }
}

kernel void random_short (const usize n, global ulong* seed, global short* out, const short origin, const short delta) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) { 
        short v = (short)next(15, &seed[get_global_id(0)]);
        out[i] = (v % delta) + origin;
    }
}

kernel void random_int (const usize n, global ulong* seed, global int* out, const int origin, const int delta) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) { 
        int v = (int)next(31, &seed[get_global_id(0)]);
        out[i] = (v % delta) + origin;
    }
}

kernel void random_long (const usize n, global ulong* seed, global long* out, const long origin, const long delta) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        long v = ((long)next(31, &seed[get_global_id(0)]) << 32) | (long)next(32, &seed[get_global_id(0)]);
        out[i] = (out[i] % delta) + origin;
    }
}

#if HALF
    #define HALF_UNIT() ((half)(1 << 11));

    kernel void random_half (const usize n, global ulong* seed, global half* out, const half origin, const half delta) {
        for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
            half v = (half)next(11, &seed[get_global_id(0)]) / HALF_UNIT();
            out[i] = (v * delta) + origin;
        }
    }
#endif

kernel void random_float (const usize n, global ulong* seed, global float* out, const float origin, const float delta) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        float v = (float)next(24, &seed[get_global_id(0)]) / FLOAT_UNIT();
        out[i] = (v * delta) + origin;
    }
}

#if DOUBLE
    #define DOUBLE_UNIT() ((double)(1l << 53));

    kernel void random_double (const usize n, global ulong* seed, global double* out, const double origin, const double delta) {
        for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
            ulong bits = ((ulong)next(26, &seed[get_global_id(0)]) << 27) | (ulong)next(27, &seed[get_global_id(0)]);
            double v = (double)bits / DOUBLE_UNIT();
            out[i] = (v * delta) + origin;
        }
    }
#endif

// LOOP RANDOMS
kernel void loop_random_uchar (const usize n, global ulong* seed, global uchar* out, const uchar origin, const uchar bound) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        uchar v;
        do {
            v = (uchar)next(8, &seed[get_global_id(0)]);
        } while (v < origin || v > bound);

        out[i] = v;
    }
}

kernel void loop_random_ushort (const usize n, global ulong* seed, global ushort* out, const ushort origin, const ushort bound) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        ushort v;
        do {
            v = (ushort)next(16, &seed[get_global_id(0)]);
        } while (v < origin || v > bound);

        out[i] = v;
    }
}

kernel void loop_random_uint (const usize n, global ulong* seed, global uint* out, const uint origin, const uint bound) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        uint v;
        do {
            v = next(32, &seed[get_global_id(0)]);
        } while (v < origin || v > bound);

        out[i] = v;
    }
}

kernel void loop_random_ulong (const usize n, global ulong* seed, global ulong* out, const ulong origin, const ulong bound) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        ulong v;
        do {
            v = ((ulong)next(32, &seed[get_global_id(0)]) << 32) | (ulong)next(32, &seed[get_global_id(0)]);
        } while (v < origin || v > bound);

        out[i] = v;
    }
}

kernel void loop_random_char (const usize n, global ulong* seed, global uchar* out, const char origin, const char bound) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        char v;
        do {
            v = (char)next(8, &seed[get_global_id(0)]);
        } while (v < origin || v > bound);

        out[i] = v;
    }
}

kernel void loop_random_short (const usize n, global ulong* seed, global ushort* out, const short origin, const short bound) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        short v;
        do {
            v = (short)next(16, &seed[get_global_id(0)]);
        } while (v < origin || v > bound);

        out[i] = v;
    }
}

kernel void loop_random_int (const usize n, global ulong* seed, global int* out, const int origin, const int bound) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        int v;
        do {
            v = (int)next(32, &seed[get_global_id(0)]);
        } while (v < origin || v > bound);

        out[i] = v;
    }
}

kernel void loop_random_long (const usize n, global ulong* seed, global long* out, const long origin, const long bound) {
    for (usize i = get_global_id(0); i < n; i += get_global_size(0)) {
        long v;
        do {
            v = ((long)next(32, &seed[get_global_id(0)]) << 32) | (long)next(32, &seed[get_global_id(0)]);
        } while (v < origin || v > bound);

        out[i] = v;
    }
}
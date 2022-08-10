#ifndef PRECISION
    typedef float real;
    typedef uint usize;
#endif

typedef struct {
    global real* ptr;
    usize len;
} slice;

kernel void quicksort_kernel (const usize n, global real* values) {

}

static void quicksort (const slice slice) {
    if (slice.len <= 1) return;
    quicksort_kernel(slice.len, slice.ptr);
}
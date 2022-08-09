#define NEW_ATOMICS __OPENCL_C_VERSION__ >= 300

#if NEW_ATOMICS
    #define INIT_ATOMIC(x) ATOMIC_VAR_INIT(x)
#else
    #define INIT_ATOMIC(x) x
    typedef int atomic_int;
    typedef uint atomic_uint; 
#endif

static inline atomic_int_init (volatile atomic_uin)
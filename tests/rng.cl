#define MUTPILIER 0x5DEECE66Dl
#define ADDEND 0xBl
#define MASK() ((1l << 48) - 1)

global atomic_ulong SEED = ATOMIC_VAR_INIT((8682522807148012l * 1181783497276652981L) ^ TIME);

kernel void next_bytes (const uint n, global uchar* out) {
    const uint ID = get_global_id(0);
    const uint SIZE = get_global_size(0);
    ulong oldseed, nextseed;

    for (uint i = ID; i < n; i += SIZE) {
        do {
            oldseed = atomic_load(&SEED);
            nextseed = (oldseed * MUTPILIER + ADDEND) & MASK();
        } while (!atomic_compare_exchange_strong(&SEED, &oldseed, nextseed));
        out[i] = nextseed >> 40;
    }
}
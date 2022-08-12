// Source: https://github.com/Gram21/GPUSorting/blob/master/Code/Sort.cl

#ifndef MAX_LOCAL_SIZE
    #define MAX_LOCAL_SIZE 256 // set via compile options
#endif

#ifndef PRECISION
    typedef float real;
    typedef uint usize;
#endif

static inline char compare (const uchar desc, real a, real b) {
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
		w.a = a;
		bits left = w.b;

		// Get right bits
		w.a = b;
		bits right = w.b;

		left ^= (bits)((ubits)(left >> (PRECISION - 1)) >> 1);
		right ^= (bits)((ubits)(right >> (PRECISION - 1)) >> 1);

		if (desc != 0) return left > right;
		return left < right;
	#else
		if (desc != 0) return a > b;
		return a < b;
	#endif
} 

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// needed helper methods
static inline void swap(real *a, real *b) {
	real tmp;
	tmp = *b;
	*b = *a;
	*a = tmp;
}

// dir == 1 means ascending
static inline void sort(const uchar desc, real *a, real *b, char dir) {
	if (compare(desc, *a, *b) == dir) swap(a, b);
}

static inline void swapLocal(__local real *a, __local real *b) {
	real tmp;
	tmp = *b;
	*b = *a;
	*a = tmp;
}

// dir == 1 means ascending
static inline void sortLocal(const uchar desc, __local real *a, __local real *b, char dir) {
	if (compare(desc, *a, *b) == dir) swapLocal(a, b);
}

__kernel void Sort_BitonicMergesortStart(const uchar desc, const __global real* inArray, __global real* outArray) {
	__local real local_buffer[MAX_LOCAL_SIZE * 2];
	const usize gid = get_global_id(0);
	const usize lid = get_local_id(0);

	usize index = get_group_id(0) * (MAX_LOCAL_SIZE * 2) + lid;
	//load into local mem
	local_buffer[lid] = inArray[index];
	local_buffer[lid + MAX_LOCAL_SIZE] = inArray[index + MAX_LOCAL_SIZE];

	usize clampedGID = gid & (MAX_LOCAL_SIZE - 1);

	// bitonic merge
	for (usize blocksize = 2; blocksize < MAX_LOCAL_SIZE * 2; blocksize <<= 1) {
		char dir = (clampedGID & (blocksize / 2)) == 0; // sort every other block in the other direction (faster % calc)
#pragma unroll
		for (usize stride = blocksize >> 1; stride > 0; stride >>= 1){
			barrier(CLK_LOCAL_MEM_FENCE);
			usize idx = 2 * lid - (lid & (stride - 1)); //take every other input BUT starting neighbouring within one block
			sortLocal(desc, &local_buffer[idx], &local_buffer[idx + stride], dir);
		}
	}

	// bitonic merge for biggest group is special (unrolling this so we dont need ifs in the part above)
	char dir = (clampedGID & 0); //even or odd? sort accordingly
#pragma unroll
	for (usize stride = MAX_LOCAL_SIZE; stride > 0; stride >>= 1){
		barrier(CLK_LOCAL_MEM_FENCE);
		usize idx = 2 * lid - (lid & (stride - 1));
		sortLocal(desc, &local_buffer[idx], &local_buffer[idx + stride], dir);
	}

	// sync and write back
	barrier(CLK_LOCAL_MEM_FENCE);
	outArray[index] = local_buffer[lid];
	outArray[index + MAX_LOCAL_SIZE] = local_buffer[lid + MAX_LOCAL_SIZE];
}

__kernel void Sort_BitonicMergesortLocal(const uchar desc, __global real* data, const usize size, const usize blocksize, usize stride)
{
	// This Kernel is basically the same as Sort_BitonicMergesortStart except of the "unrolled" part and the provided parameters
	__local real local_buffer[2 * MAX_LOCAL_SIZE];
	usize gid = get_global_id(0);
	usize groupId = get_group_id(0);
	usize lid = get_local_id(0);
	usize clampedGID = gid & (size / 2 - 1);

	usize index = groupId * (MAX_LOCAL_SIZE * 2) + lid;
	//load into local mem
	local_buffer[lid] = data[index];
	local_buffer[lid + MAX_LOCAL_SIZE] = data[index + MAX_LOCAL_SIZE];

	// bitonic merge
	char dir = (clampedGID & (blocksize / 2)) == 0; //same as above, % calc
#pragma unroll
	for (; stride > 0; stride >>= 1) {
		barrier(CLK_LOCAL_MEM_FENCE);
		usize idx = 2 * lid - (lid & (stride - 1));
		sortLocal(desc, &local_buffer[idx], &local_buffer[idx + stride], dir);
	}

	// sync and write back
	barrier(CLK_LOCAL_MEM_FENCE);
	data[index] = local_buffer[lid];
	data[index + MAX_LOCAL_SIZE] = local_buffer[lid + MAX_LOCAL_SIZE];
}

__kernel void Sort_BitonicMergesortGlobal(const uchar desc, __global real* data, const usize size, const usize blocksize, const usize stride)
{
	// TO DO: Kernel implementation
	usize gid = get_global_id(0);
	usize clampedGID = gid & (size / 2 - 1);

	//calculate index and dir like above
	usize index = 2 * clampedGID - (clampedGID & (stride - 1));
	char dir = (clampedGID & (blocksize / 2)) == 0; //same as above, % calc

	//bitonic merge
	real left = data[index];
	real right = data[index + stride];

	sort(desc, &left, &right, dir);

	// writeback
	data[index] = left;
	data[index + stride] = right;
}
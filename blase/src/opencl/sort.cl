// Source: https://github.com/Gram21/GPUSorting/blob/master/Code/Sort.cl

#ifndef MAX_LOCAL_SIZE
    typedef float real;
    typedef uint usize;
    #define MAX_LOCAL_SIZE 256 // set via compile options
#endif

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// needed helper methods
static inline void swap(real *a, real *b) {
	uint tmp;
	tmp = *b;
	*b = *a;
	*a = tmp;
}

// dir == 1 means ascending
static inline void sort(real *a, real *b, char dir) {
	if ((*a > *b) == dir) swap(a, b);
}

static inline void swapLocal(__local real *a, __local real *b) {
	uint tmp;
	tmp = *b;
	*b = *a;
	*a = tmp;
}

// dir == 1 means ascending
static inline void sortLocal(__local real *a, __local real *b, char dir) {
	if ((*a > *b) == dir) swapLocal(a, b);
}

__kernel void Sort_BitonicMergesortStart(const __global real* inArray, __global real* outArray) {
	__local real local_buffer[MAX_LOCAL_SIZE * 2];
	const usize gid = get_global_id(0);
	const usize lid = get_local_id(0);

	usize index = get_group_id(0) * (MAX_LOCAL_SIZE * 2) + lid;
	//load into local mem
	local_buffer[lid] = inArray[index];
	local_buffer[lid + MAX_LOCAL_SIZE] = inArray[index + MAX_LOCAL_SIZE];

	usize clampedGID = gid & (MAX_LOCAL_SIZE - 1);

	// bitonic merge
	for (uint blocksize = 2; blocksize < MAX_LOCAL_SIZE * 2; blocksize <<= 1) {
		char dir = (clampedGID & (blocksize / 2)) == 0; // sort every other block in the other direction (faster % calc)
#pragma unroll
		for (usize stride = blocksize >> 1; stride > 0; stride >>= 1){
			barrier(CLK_LOCAL_MEM_FENCE);
			usize idx = 2 * lid - (lid & (stride - 1)); //take every other input BUT starting neighbouring within one block
			sortLocal(&local_buffer[idx], &local_buffer[idx + stride], dir);
		}
	}

	// bitonic merge for biggest group is special (unrolling this so we dont need ifs in the part above)
	char dir = (clampedGID & 0); //even or odd? sort accordingly
#pragma unroll
	for (usize stride = MAX_LOCAL_SIZE; stride > 0; stride >>= 1){
		barrier(CLK_LOCAL_MEM_FENCE);
		usize idx = 2 * lid - (lid & (stride - 1));
		sortLocal(&local_buffer[idx], &local_buffer[idx + stride], dir);
	}

	// sync and write back
	barrier(CLK_LOCAL_MEM_FENCE);
	outArray[index] = local_buffer[lid];
	outArray[index + MAX_LOCAL_SIZE] = local_buffer[lid + MAX_LOCAL_SIZE];
}

__kernel void Sort_BitonicMergesortLocal(__global real* data, const usize size, const usize blocksize, usize stride)
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
		sortLocal(&local_buffer[idx], &local_buffer[idx + stride], dir);
	}

	// sync and write back
	barrier(CLK_LOCAL_MEM_FENCE);
	data[index] = local_buffer[lid];
	data[index + MAX_LOCAL_SIZE] = local_buffer[lid + MAX_LOCAL_SIZE];
}

__kernel void Sort_BitonicMergesortGlobal(__global real* data, const usize size, const usize blocksize, const usize stride)
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

	sort(&left, &right, dir);

	// writeback
	data[index] = left;
	data[index + stride] = right;
}
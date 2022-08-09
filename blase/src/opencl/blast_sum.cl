// Parameters set by the tuner or by the database. Here they are given a basic default value in case
// this kernel file is used outside of the CLBlast library.
#ifndef WGS1
  #define WGS1 64     // The local work-group size of the main kernel
  typedef uint real;
#endif
#ifndef WGS2
  #define WGS2 64     // The local work-group size of the epilogue kernel
#endif

// =================================================================================================
// The main reduction kernel, performing the loading and the majority of the operation
__kernel __attribute__((reqd_work_group_size(WGS1, 1, 1)))
void Xasum(const int n,
           const __global real* restrict xgm, const int x_offset, const int x_inc,
           __global real* output) {
  __local real lm[WGS1];
  const int lid = get_local_id(0);
  const int wgid = get_group_id(0);
  const int num_groups = get_num_groups(0);
  // Performs loading and the first steps of the reduction
  real acc = 0;
  int id = wgid*WGS1 + lid;
  while (id < n) {
    real x = xgm[id*x_inc + x_offset];
    acc += x;
    id += WGS1*num_groups;
  }
  lm[lid] = acc;
  barrier(CLK_LOCAL_MEM_FENCE);
  // Performs reduction in local memory
  for (int s=WGS1/2; s>0; s=s>>1) {
    if (lid < s) {
      lm[lid] += lm[lid + s];
    }
    barrier(CLK_LOCAL_MEM_FENCE);
  }
  // Stores the per-workgroup result
  if (lid == 0) {
    output[wgid] = lm[0];
  }
}
// =================================================================================================
// The epilogue reduction kernel, performing the final bit of the operation. This kernel has to
// be launched with a single workgroup only.
__kernel __attribute__((reqd_work_group_size(WGS2, 1, 1)))
void XasumEpilogue(const __global real* restrict input,
                   __global real* asum, const int asum_offset) {
  __local real lm[WGS2];
  const int lid = get_local_id(0);
  // Performs the first step of the reduction while loading the data
  lm[lid] = input[lid] + input[lid + WGS2];
  barrier(CLK_LOCAL_MEM_FENCE);
  // Performs reduction in local memory
  for (int s=WGS2/2; s>0; s=s>>1) {
    if (lid < s) {
      lm[lid] += lm[lid + s];
    }
    barrier(CLK_LOCAL_MEM_FENCE);
  }
  // Computes the absolute value and stores the final result
  if (lid == 0) {
    asum[asum_offset] = lm[0];
  }
}
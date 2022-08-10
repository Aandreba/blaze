// Parameters set by the tuner or by the database. Here they are given a basic default value in case
// this kernel file is used outside of the CLBlast library.
#ifndef WGS1
  #define WGS1 64     // The local work-group size of the main kernel
  #define FMA(a,b,c) (a * b) + c
  typedef uint real;
#endif
#ifndef WGS2
  #define WGS2 64     // The local work-group size of the epilogue kernel
#endif

// =================================================================================================
// The main reduction kernel, performing the loading and the majority of the operation
__kernel __attribute__((reqd_work_group_size(WGS1, 1, 1)))
void Xdot(const int n,
           const __global real* restrict xgm,
           const __global real* restrict ygm,
           __global real* output) {
  __local real lm[WGS1];
  const int lid = get_local_id(0);
  const int wgid = get_group_id(0);
  const int num_groups = get_num_groups(0);
  // Performs loading and the first steps of the reduction
  real acc = 0;
  int id = wgid*WGS1 + lid;
  while (id < n) {
    acc = FMA(xgm[id], ygm[id], acc);
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
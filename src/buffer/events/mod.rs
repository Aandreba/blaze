use blaze_proc::docfg;
flat_mod!(read, write, copy, map);

#[docfg(feature = "cl1_2")]
flat_mod!(fill);
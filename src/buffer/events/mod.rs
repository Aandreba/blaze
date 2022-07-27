use blaze_proc::docfg;
flat_mod!(read, write, copy);

#[docfg(feature = "cl1_2")]
flat_mod!(fill);
#[docfg(feature = "map")]
flat_mod!(map);
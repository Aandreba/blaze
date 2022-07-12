use rscl_proc::docfg;

flat_mod!(read, write, copy, fill);

#[docfg(feature = "map")]
flat_mod!(map);
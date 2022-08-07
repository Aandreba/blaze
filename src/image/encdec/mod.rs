flat_mod!(deflate);

use std::io::Read;

pub struct BmpDecode<R: Read> {
    inner: R
}
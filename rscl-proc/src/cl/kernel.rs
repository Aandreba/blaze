use crate::cl::Reader;

use super::{ClParse, Argument};

#[derive(Debug)]
pub struct Kernel<'a> {
    name: &'a str,
    args: Vec<Argument<'a>>
}

impl<'a> ClParse<'a> for Kernel<'a> {
    fn parse (buff: &mut super::Reader<'a>) -> Self {
        buff.next_assert_any(&["kernel", "__kernel"]);
        buff.next_assert_any(&["void"]);

        let name = buff.next();
        buff.skip_until('(', false);
        let mut args = Vec::new();
        
        while buff.peek_char() != ')' {
            args.push(buff.parse_next());
            println!("{buff:?}")
        }

        Self { name, args }
    }
}

#[test]
fn test () {
    let mut parser = Reader::new("kernel void add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
        for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
            out[id] = in[id] + rhs[id];
        }
    }");

    let kernel : Kernel = parser.parse_next();
    panic!("{kernel:?}");
}
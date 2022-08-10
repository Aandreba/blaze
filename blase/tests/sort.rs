use std::{collections::VecDeque, ops::{BitAnd, Shr, Shl}, f32::consts::PI};
use num_traits::{NumOps, Unsigned, One};

fn radix_sort <A, F: Fn(&A) -> T, T: Copy + BitAnd<T, Output = T> + Shr<usize, Output = T> + PartialEq> (mut result: Vec<A>, one: T, f: F) -> Vec<A> {
    let bits = 8 * core::mem::size_of::<T>();

    for i in 0..bits {
        let mut left = Vec::with_capacity(result.len());
        let mut right = Vec::with_capacity(result.len());

        for v in result.into_iter() {
            if (f(&v) >> i & one) == one {
                right.push(v);
            } else {
                left.push(v);
            }
        }

        left.extend(right);
        result = left;
    }

    result
}

fn radix_float (v: Vec<f32>) {
    let by_parts = v.into_iter().map(|x| {
        let bits = x.to_bits();
        let exp = ((bits >> 23) & 0xff) as u8;
        let mant = bits & 0x7FFFFF;
        (x, exp, mant)
    }).collect::<Vec<_>>();

    let radixed = radix_sort(by_parts, 1, |x| x.2);
    let radixed = radix_sort(radixed, 1, |x| x.1);
    todo!()
}

#[test]
fn test () {
    let mut v = [10, 9, 10, 2, 8, 2, 3, 2, 2, 10];
    let f = [10.0, 9.0, 10.0, 2.0, 8.0, 2.0, PI, 20.0, 2.0, 100.0];

    let sort = radix_float(f.to_vec());
    println!("{v:?}");
}

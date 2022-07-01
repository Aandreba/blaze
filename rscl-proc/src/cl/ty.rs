use syn::{custom_keyword, parse::Parse, token::{Star}};

/*
    kernel void add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
        for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
            int two = (int)in[id];
            out[id] = in[id] + rhs[id];
        }
    }
*/

custom_keyword!(unsigned);
custom_keyword!(void);
custom_keyword!(bool);
custom_keyword!(char);
custom_keyword!(uchar);
custom_keyword!(short);
custom_keyword!(ushort);
custom_keyword!(int);
custom_keyword!(uint);
custom_keyword!(long);
custom_keyword!(ulong);
custom_keyword!(float);
custom_keyword!(double);

#[derive(Debug)]
pub enum Type {
    Void,
    Bool,
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    Float,
    Double,
    Pointer (Box<Type>)
}

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let v;
        if peek_and_parse!(unsigned in input) {

            if peek_and_parse!(char in input) {
                v = Self::UChar;
            }

            else if peek_and_parse!(short in input) {
                v = Self::UShort;
            }

            else if peek_and_parse!(int in input) {
                v = Self::UInt;
            }

            else if peek_and_parse!(long in input) {
                v = Self::ULong;
            }

            else {
                return Err(syn::Error::new(input.span(), "invalid type"))
            }
        } else {
            if peek_and_parse!(void in input) {
                v = Self::Void;
            }

            else if peek_and_parse!(bool in input) {
                v = Self::Bool;
            }

            else if peek_and_parse!(char in input) {
                v = Self::Char;
            }

            else if peek_and_parse!(uchar in input) {
                v = Self::UChar;
            }

            else if peek_and_parse!(short in input) {
                v = Self::Short;
            }

            else if peek_and_parse!(ushort in input) {
                v = Self::UShort;
            }

            else if peek_and_parse!(int in input) {
                v = Self::Int;
            }

            else if peek_and_parse!(uint in input) {
                v = Self::UInt;
            }

            else if peek_and_parse!(long in input) {
                v = Self::Long;
            }

            else if peek_and_parse!(ulong in input) {
                v = Self::ULong;
            }

            else if peek_and_parse!(float in input) {
                v = Self::Float;
            }

            else if peek_and_parse!(double in input) {
                v = Self::Double;
            }

            else {
                return Err(syn::Error::new(input.span(), "invalid type"))
            }
        }

        if peek_and_parse!(Star in input) {
            return Ok(Self::Pointer(Box::new(v)))
        }

        return Ok(v)
    }
}
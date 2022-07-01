use super::ClParse;

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

impl ClParse<'_> for Type {
    fn parse (buff: &mut super::Reader) -> Self {    
        println!("{buff:?}");   
        let mut next = buff.next();

        if next == "unsigned" {
            let mut next = buff.next();
            let pointer = next.ends_with('*');

            if pointer {
                next = &next[..next.len() - 1]
            }

            let v = match next {
                "char" => Self::UChar,
                "short" => Self::UShort,
                "int" => Self::UInt,
                "long" => Self::ULong,
                other => panic!("invalid type 'unsigned {other}'")
            };

            return match pointer {
                true => Self::Pointer(Box::new(v)),
                _ => v
            }
        }

        let pointer = next.ends_with('*');
        if pointer {
            next = &next[..next.len() - 1]
        }
        
        let v = match next {
            "void" => Self::Void,
            "bool" => Self::Bool,
            "uchar" => Self::UChar,
            "char" => Self::Char,
            "ushort" => Self::UShort,
            "short" => Self::Short,
            "int" => Self::Int,
            "uint" => Self::UInt,
            "ulong" => Self::ULong,
            "long" => Self::Long,
            "float" => Self::Float,
            "double" => Self::Double,
            other => panic!("invalid type '{other}'")
        };

        match pointer {
            true => Self::Pointer(Box::new(v)),
            _ => v
        }
    }
}
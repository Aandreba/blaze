use super::{Access, ClParse, Type};

#[derive(Debug)]
pub struct Argument<'a> {
    access: Access,
    constness: bool,
    ty: Type,
    name: &'a str
}

impl<'a> ClParse<'a> for Argument<'a> {
    #[inline(always)]
    fn parse (buff: &mut super::Reader<'a>) -> Self {
        let constness = buff.peek() == "const";
        if constness { buff.skip_until(char::is_whitespace, false) }
        
        let access = buff.parse_next();
        let ty = buff.parse_next();
        let name = buff.next();

        Self { constness, access, ty, name }
    }
}
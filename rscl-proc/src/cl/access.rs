use super::ClParse;

#[derive(Debug)]
pub enum Access {
    Global,
    Local,
    Const,
    Private   
}

impl<'a> ClParse<'a> for Access {
    fn parse (buff: &mut super::Reader<'a>) -> Self {
        match buff.peek() {
            "global" | "__global" => {
                buff.skip_until(char::is_whitespace, false);
                Self::Global
            },

            "local" | "__local" => {
                buff.skip_until(char::is_whitespace, false);
                Self::Local
            }

            "constant" | "__constant" => {
                buff.skip_until(char::is_whitespace, false);
                Self::Const
            }
            
            _ => Self::Private
        }
    }
}
use std::{str::{SplitWhitespace, Chars, pattern::Pattern}, iter::Peekable, ops::{Range}, fmt::Debug};
use derive_syn_parse::Parse;
use proc_macro2::{Ident, TokenStream};
use syn::{Token, LitStr};

flat_mod!(ty, kernel, access, arg);

pub fn rscl (rscl: Rscl) -> TokenStream {
    let value = rscl.program.value();
    let mut parser = Reader::new(&value);
    let kernel : Kernel = parser.parse_next();
    
    panic!("{kernel:?}");
    todo!()
}

#[derive(Parse)]
pub struct Rscl {
    pub ident: Ident,
    pub at_token: Token![@],
    pub program: LitStr
}

pub trait ClParse<'a>: Sized {
    fn parse (buff: &mut Reader<'a>) -> Self;
}

pub struct Reader<'a> {
    str: &'a str,
    idx: usize
}

impl<'a> Reader<'a> {
    #[inline]
    pub fn new (str: &'a str) -> Self {
        Self { str, idx: 0 }
    }

    #[inline(always)]
    pub fn parse_next<T> (&mut self) -> T where T: ClParse<'a> {
        <T as ClParse<'a>>::parse(self)
    } 

    #[inline(always)]
    pub fn next_until (&mut self, predicate: impl Clone + Pattern<'a>, contain: Containment) -> &'a str {
        let chars = self.str[self.idx..].char_indices();
        
        for (mut idx, _) in chars {
            idx += self.idx;

            if predicate.clone().is_contained_in(&self.str[self.idx..=idx]) {
                let result;

                match contain {
                    Containment::Exclude => {
                        result = &self.str[self.idx..idx];
                        self.idx = idx
                    },

                    Containment::Include => {
                        result = &self.str[self.idx..=idx];
                        self.idx = idx + 1
                    },

                    Containment::Skip => {
                        result = &self.str[self.idx..idx];
                        self.idx = idx + 1
                    },
                }

                return result.trim()
            }
        }

        todo!()
    }

    #[inline(always)]
    pub fn peek_until (&mut self, predicate: impl Clone + Pattern<'a>, exclude: bool) -> &'a str {
        let chars = self.str[self.idx..].char_indices();
        
        for (mut idx, _) in chars {
            idx += self.idx;

            if predicate.clone().is_contained_in(&self.str[self.idx..=idx]) {
                let v = match exclude {
                    true => &self.str[self.idx..idx],
                    _ => &self.str[self.idx..=idx],
                };

                return v.trim();
            }
        }

        todo!()
    }

    #[inline(always)]
    pub fn skip_until (&mut self, predicate: impl Clone + Pattern<'a>, exclude: bool) {
        let chars = self.str[self.idx..].char_indices();
        
        for (mut idx, _) in chars {
            idx += self.idx;

            if predicate.clone().is_contained_in(&self.str[self.idx..=idx]) {
                self.idx = match exclude {
                    true => idx,
                    _ => idx + 1
                };

                return
            }
        }

        todo!()
    }

    #[inline(always)]
    pub fn next (&mut self) -> &'a str {
        self.next_until(char::is_whitespace, Containment::Skip)
    }

    #[inline(always)]
    pub fn peek (&mut self) -> &'a str {
        self.peek_until(char::is_whitespace, true)
    }

    #[inline(always)]
    pub fn peek_char (&self) -> char {
        self.str[self.idx..].chars().next().expect("No more tokens to parse")
    }

    #[inline(always)]
    pub fn skip (&mut self, n: usize) {
        self.idx += n
    }

    #[inline(always)]
    pub fn next_assert_any (&mut self, pat: &[&str]) {
        let next = self.next();

        for pat in pat {
            if next == *pat { return; }
        }

        panic!("No matches found: {pat:?}")
    }
}

impl Debug for Reader<'_> {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.str[self.idx..], f)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Containment {
    Include,
    Exclude,
    Skip
}
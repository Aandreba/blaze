use derive_syn_parse::Parse;
use super::{signature::Signature, expr::Block};

#[derive(Parse)]
pub struct Kernel {
    pub sig: Signature,
    pub block: Block
}
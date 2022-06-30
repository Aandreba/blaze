use derive_syn_parse::Parse;
use super::signature::Signature;

#[derive(Parse)]
pub struct Kernel {
    pub sig: Signature
}
use super::expr::Block;

#[inline(always)]
pub fn compile (block: Block) -> String {
    block.to_string()
}
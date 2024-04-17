use crate::block::Block;
use std::ops::Deref;

pub struct BlockStatement {
    block: Block,
}

impl Deref for BlockStatement {
    type Target = Block;

    fn deref(&self) -> &Block {
        &self.block
    }
}

use std::ops::Deref;

use crate::block::Block;

pub struct BlockStatement {
    block: Block,
}

impl Deref for BlockStatement {
    type Target = Block;

    fn deref(&self) -> &Block {
        &self.block
    }
}

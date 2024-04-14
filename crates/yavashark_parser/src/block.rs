use crate::declaration::Declaration;
use crate::statement::Statement;

pub struct Block {
    statements: Vec<StatementList>
}

struct StatementList {
    list: Option<Box<StatementList>>,
    item: StatementListItem,
}

enum StatementListItem {
    Statement(Statement),
    Declaration(Declaration),
}
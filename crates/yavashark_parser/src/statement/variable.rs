pub struct VariableStatement {
    //var
    decl: VariableDeclarationList
}

pub struct VariableDeclarationList {
    list: Option<Box<VariableDeclarationList>>,
    item: VariableDeclaration,
}

pub enum VariableDeclaration {
    Identifier(BindingIdentifier),
    IdentifierInitializer(BindingIdentifier, Initializer),
    Pattern(Pattern, Initializer)
}

struct Initializer {
    // =
    expr: AssigmentExpression,
}


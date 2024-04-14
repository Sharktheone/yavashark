pub enum Identifier {
    //TODO: spec is a mess...    
}

pub enum IdentifierReference {
    Identifier(Identifier),
    Yield,
    Await,
}

pub enum BindingIdentifier {
    Identifier(Identifier),
    Yield,
    Await,
}

pub enum LabelIdentifier {
    Identifier(Identifier),
    Yield,
    Await,
}
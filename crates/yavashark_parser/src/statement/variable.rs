use crate::expression::assigment::AssigmentExpression;
use crate::identifier::Identifier;

pub struct VariableStatement {
    //var
    decl: VariableDeclarationList,
}

pub struct VariableDeclarationList {
    list: Option<Box<VariableDeclarationList>>,
    item: VariableDeclaration,
}

pub enum VariableDeclaration {
    Identifier(BindingIdentifier),
    IdentifierInitializer(BindingIdentifier, Initializer),
    Pattern(Pattern, Initializer),
}

pub struct Initializer {
    // =
    expr: AssigmentExpression,
}

pub enum BindingIdentifier {
    Identifier(Identifier),
    Yield,
    Await,
}

pub enum Pattern {
    ObjectBindingPattern(ObjectBindingPattern),
    ArrayBindingPattern(ArrayBindingPattern),
}


pub struct ArrayBindingPattern;

pub enum ObjectBindingPattern {
    // { }
    Empty,
    // { BindingRestProperty }
    BindingRestProperty(BindingRestProperty),
    // { BindingPropertyList }
    BindingPropertyList(BindingPropertyList),
    // { BindingPropertyList , }
    BindingPropertyListRest(BindingPropertyList, BindingRestProperty),
}

pub struct BindingRestProperty {
    // ... BindingIdentifier
    ident: BindingIdentifier,
}

pub struct BindingPropertyList {
    property: BindingProperty,
    list: Option<Box<BindingPropertyList>>,
}

pub enum BindingProperty {
    // SingleNameBinding
    SingleNameBinding(SingleNameBinding),
    // PropertyName : BindingElement
    PropertyNameBindingElement(PropertyName, BindingElement),
}

pub struct BindingElement;

pub struct SingleNameBinding {
    // BindingIdentifier
    ident: BindingIdentifier,
    // Initializer
    init: Initializer,
}

pub struct PropertyName {
    //TODO
}

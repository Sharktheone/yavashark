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

struct Initializer {
    // =
    expr: AssigmentExpression,
}

enum BindingIdentifier {
    Identifier(Identifier),
    Yield,
    Await,
}

enum Pattern {
    ObjectBindingPattern(ObjectBindingPattern),
    ArrayBindingPattern(ArrayBindingPattern),
}

enum ObjectBindingPattern {
    // { }
    Empty,
    // { BindingRestProperty }
    BindingRestProperty(BindingRestProperty),
    // { BindingPropertyList }
    BindingPropertyList(BindingPropertyList),
    // { BindingPropertyList , }
    BindingPropertyListRest(BindingPropertyList, BindingRestProperty),
}

struct BindingRestProperty {
    // ... BindingIdentifier
    ident: BindingIdentifier,
}

struct BindingPropertyList {
    property: BindingProperty,
    list: Option<Box<BindingPropertyList>>,
}

enum BindingProperty {
    // SingleNameBinding
    SingleNameBinding(SingleNameBinding),
    // PropertyName : BindingElement
    PropertyNameBindingElement(PropertyName, BindingElement),
}

struct SingleNameBinding {
    // BindingIdentifier
    ident: BindingIdentifier,
    // Initializer
    init: Initializer,
}

struct PropertyName {
//TODO
}
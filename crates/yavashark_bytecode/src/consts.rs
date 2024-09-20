



#[derive(Debug, Clone)]
pub enum ConstValue {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(ObjectLiteralBlueprint),
    Symbol(String),
}

#[derive(Debug, Clone)]
pub struct ObjectLiteralBlueprint {}

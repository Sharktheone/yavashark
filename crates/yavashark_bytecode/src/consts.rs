pub enum ConstValue {
    Null,
    Undefined,
    Number(f64),
    String(String),
    Boolean(bool),
    Object(ObjectLiteralBlueprint),
    Symbol(String),
}

pub struct ObjectLiteralBlueprint {}

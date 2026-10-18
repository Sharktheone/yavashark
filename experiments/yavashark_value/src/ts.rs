enum Value<T: TSValue> {
    Null,
    Undefined,
    Some(T),
}

impl<T: TSValue> Value<T> {
    pub fn new(value: T) -> Self {
        Value::Some(value)
    }

    pub fn type_of(&self) -> &'static str {
        match self {
            Value::Null => "object",
            Value::Undefined => "undefined",
            Value::Some(value) => value.type_of(),
        }
    }

    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Undefined => ValueType::Undefined,
            Value::Some(value) => ValueType::Type(value.value_type()),
        }
    }
}

pub trait TSValue {
    fn type_of(&self) -> &'static str;
    fn value_type(&self) -> TSValueType;
}

pub enum TSValueType {
    Number,
    String,
    Boolean,
    Object,
    Array,
    Function,
}

pub enum ValueType {
    Null,
    Undefined,
    Type(TSValueType),
}

use std::collections::HashMap;

struct Scope {
    parent: Option<Box<Scope>>,
    variables: HashMap<String, Value>,
}
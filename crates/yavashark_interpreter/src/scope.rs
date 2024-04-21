use std::collections::HashMap;
use yavashark_value::Value;

pub(crate) struct Scope<'ctx> {
    parent: Option<&'ctx mut Scope<'ctx>>,
    variables: HashMap<String, Value>,
}
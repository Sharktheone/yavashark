use proc_macro2::{Ident, Span};
use syn::{Path, PathSegment};

#[allow(unused)]
pub struct Config {
    pub env_path: Path,
    pub value_path: Path,
    pub realm: Path,
    pub error: Path,
    pub native_function: Path,
    pub native_constructor: Path,
    pub variable: Path,
    pub object_handle: Path,
    pub object: Path,
    pub value: Path,
    pub value_result: Path,
    pub object_property: Path,
    pub try_into_value: Path,
    pub mut_object: Path,
    pub mut_obj: Path,
    pub extractor: Path,
    pub extract_value: Path,
    pub from_value_output: Path,
}

impl Config {
    pub fn new(span: Span) -> Self {
        let env_path = crate::env_path();
        let value_path = crate::value_path();

        let mut realm = env_path.clone();
        realm
            .segments
            .push(PathSegment::from(Ident::new("Realm", span)));

        let mut error = env_path.clone();
        error
            .segments
            .push(PathSegment::from(Ident::new("Error", span)));

        let mut native_function = env_path.clone();
        native_function
            .segments
            .push(PathSegment::from(Ident::new("NativeFunction", span)));

        let mut native_constructor = env_path.clone();
        native_constructor
            .segments
            .push(PathSegment::from(Ident::new("NativeConstructor", span)));

        let mut variable = env_path.clone();
        variable
            .segments
            .push(PathSegment::from(Ident::new("Variable", span)));

        let mut object_handle = env_path.clone();
        object_handle
            .segments
            .push(PathSegment::from(Ident::new("ObjectHandle", span)));

        let mut object = env_path.clone();
        object
            .segments
            .push(PathSegment::from(Ident::new("object", span)));
        object
            .segments
            .push(PathSegment::from(Ident::new("Object", span)));

        let mut value = env_path.clone();
        value
            .segments
            .push(PathSegment::from(Ident::new("Value", span)));

        let mut value_result = env_path.clone();
        value_result
            .segments
            .push(PathSegment::from(Ident::new("ValueResult", span)));

        let mut object_property = env_path.clone();
        object_property
            .segments
            .push(PathSegment::from(Ident::new("ObjectProperty", span)));

        let mut try_into_value = env_path.clone();
        try_into_value
            .segments
            .push(PathSegment::from(Ident::new("conversion", span)));
        try_into_value
            .segments
            .push(PathSegment::from(Ident::new("TryIntoValue", span)));

        let mut mut_object = env_path.clone();
        mut_object
            .segments
            .push(PathSegment::from(Ident::new("MutObject", span)));

        let mut mut_obj = value_path.clone();
        mut_obj
            .segments
            .push(PathSegment::from(Ident::new("MutObj", span)));

        let mut extractor = env_path.clone();
        extractor
            .segments
            .push(PathSegment::from(Ident::new("conversion", span)));
        extractor
            .segments
            .push(PathSegment::from(Ident::new("Extractor", span)));

        let mut extract_value = env_path.clone();
        extract_value
            .segments
            .push(PathSegment::from(Ident::new("conversion", span)));
        extract_value
            .segments
            .push(PathSegment::from(Ident::new("ExtractValue", span)));

        let mut from_value_output = env_path.clone();
        from_value_output
            .segments
            .push(PathSegment::from(Ident::new("conversion", span)));
        from_value_output
            .segments
            .push(PathSegment::from(Ident::new("FromValueOutput", span)));

        Self {
            env_path,
            value_path,
            realm,
            error,
            native_function,
            native_constructor,
            variable,
            object_handle,
            object,
            value,
            value_result,
            object_property,
            try_into_value,
            mut_object,
            mut_obj,
            extractor,
            extract_value,
            from_value_output,
        }
    }
}

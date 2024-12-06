use proc_macro2::{Ident, Span};
use syn::{Path, PathSegment};

struct Config {
    crate_path: Path,
    realm: Path,
    error: Path,
    native_function: Path,
    native_constructor: Path,
    variable: Path,
    object_handle: Path,
    object: Path,
    value: Path,
}


impl Config {
    fn new(span: Span) -> Self {
        let crate_path = Path::from(Ident::new("crate", span);

        let mut realm = crate_path.clone();
        realm
            .segments
            .push(PathSegment::from(Ident::new("Realm", span));

        let mut error = crate_path.clone();
        error
            .segments
            .push(PathSegment::from(Ident::new("Error", span));

        let mut native_function = crate_path.clone();
        native_function
            .segments
            .push(PathSegment::from(Ident::new("NativeFunction", span));

        let mut native_constructor = crate_path.clone();
        native_constructor
            .segments
            .push(PathSegment::from(Ident::new(
                "NativeConstructor",
                span
            )));

        let mut variable = crate_path.clone();
        variable
            .segments
            .push(PathSegment::from(Ident::new("Variable", span));

        let mut object_handle = crate_path.clone();
        object_handle
            .segments
            .push(PathSegment::from(Ident::new("ObjectHandle", span));

        let mut object = crate_path.clone();
        object
            .segments
            .push(PathSegment::from(Ident::new("object", span));
        object
            .segments
            .push(PathSegment::from(Ident::new("Object", span));

        let mut value = crate_path.clone();
        value
            .segments
            .push(PathSegment::from(Ident::new("Value", span));
        
        
        Self {
            crate_path,
            realm,
            error,
            native_function,
            native_constructor,
            variable,
            object_handle,
            object,
            value,
        }
    }
}
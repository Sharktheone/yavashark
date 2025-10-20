use crate::{Realm, Res};

// Constructs a default value if a method is called on the Object's prototype
pub trait ProtoDefault: Sized {
    fn proto_default(realm: &mut Realm) -> Res<Self>;
    fn null_proto_default() -> Self;
}

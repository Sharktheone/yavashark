use crate::Realm;

// Constructs a default value if a method is called on the Object's prototype
pub trait ProtoDefault {
    fn proto_default(realm: &Realm) -> Self;
    fn null_proto_default() -> Self;
}

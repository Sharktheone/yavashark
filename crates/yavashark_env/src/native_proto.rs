use crate::{Error, ObjectHandle, Realm};

/// Trait for native JavaScript objects that need prototype setup in the realm
pub trait NativeProto {
    /// Get the prototype as a reference from the realm
    fn get_prototype(realm: &Realm) -> Option<&ObjectHandle>;
    
    /// Get the prototype as a mutable reference from the realm  
    fn get_prototype_mut(realm: &mut Realm) -> Option<&mut ObjectHandle>;
    
    /// Initialize and setup everything in the realm for this native type
    /// This should be called during realm initialization
    fn setup_in_realm(realm: &mut Realm) -> Result<(), Error>;
}
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::ptr::NonNull;
use yavashark_garbage::GcRef;
use crate::BoxedObj;
use crate::property_key::{InternalPropertyKey, PropertyKey};

type Value = ();
type ObjectProperty = ();
type Error = ();

type Res<T = ()> = std::result::Result<T, Error>;
type ValueResult = std::result::Result<Value, Error>;
type PropertyResult = std::result::Result<ObjectProperty, Error>;

type Realm = ();

type RealmRef<'a> = &'a mut Realm;



pub trait Object: Debug + Any + 'static {
    fn define_property(&self, name: InternalPropertyKey, value: ObjectProperty, realm: &mut Realm) -> Res;
    
    fn define_setter(&self, name: InternalPropertyKey, setter: ObjectProperty, realm: &mut Realm) -> Res;
    fn define_getter(&self, name: InternalPropertyKey, getter: ObjectProperty, realm: &mut Realm) -> Res;
    
    fn get_own_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> PropertyResult;
    
    fn get_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> PropertyResult;
    
    fn delete_property(&self, name: InternalPropertyKey) -> Res;
    
    fn has_own_key(&self, name: InternalPropertyKey) -> bool;
    fn has_key(&self, name: InternalPropertyKey) -> bool;
    
    fn properties(&self) -> Result<Vec<(PropertyKey, Value)>, Error>;
    fn keys(&self) -> Result<Vec<PropertyKey>, Error>;
    fn values(&self) -> Result<Vec<Value>, Error>;
    
    fn clear(&self) -> Res;
    
    fn call(
        &self,
        realm: &mut Realm,
        args: Vec<Value>,
        this: Value,
    ) -> ValueResult;
    
    fn is_function(&self) -> bool;
    
    fn primitive(&self) -> Option<Value> {
        None
    }
    
    fn prototype(&self) -> Value;
    fn set_prototype(&self, proto: Value) -> Res;


    // /// # Safety
    // /// This function should only return references that are actually in the object!
    // /// Else it will leak memory and cause undefined behavior, same for references that are in the object but not known to the gc!
    // unsafe fn custom_gc_refs(&self) -> Vec<GcRef<BoxedObj<Realm>>> {
    //     Vec::new()
    // }


    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    
    #[allow(unused_variables)]
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult;

    fn is_constructor(&self) -> bool {
        false
    }

    /// # Safety
    /// - Caller and implementer must ensure that the pointer is a valid pointer to the type which the type id represents
    /// - Caller and implementer must ensure that the pointer is valid for the same lifetime of self
    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else {
            None
        }
    }
    
    fn is_extensible(&self) -> bool {
        true
    }
    
    fn prevent_extensions(&self) -> Res;
    
    fn is_frozen(&self) -> bool {
        false
    }
    
    fn freeze(&self) -> Res;
    
    fn is_sealed(&self) -> bool {
        false
    }
    
    fn seal(&self) -> Res;
}
#[cfg(test)]
mod tests {
    use crate::{Object, Value, Realm, NativeProto};
    use crate::complete_example::ExampleBuiltinType;

    #[test]
    fn test_object_null_pattern() {
        // Test that Object::null() creates an object with null prototype
        let obj = Object::null();
        
        // Verify the object starts with null prototype as required
        let prototype = obj.prototype().unwrap();
        assert_eq!(prototype.value, Value::Null);
    }

    #[test]
    fn test_realm_demonstrate_null_initialization() {
        // Test the demonstration method
        let result = Realm::demonstrate_null_object_initialization();
        assert!(result.is_ok());
        
        let obj = result.unwrap();
        let prototype = obj.prototype().unwrap();
        assert_eq!(prototype.value, Value::Null);
    }

    #[test]
    fn test_native_proto_pattern() {
        // Test that the NativeProto pattern would work
        let mut realm = Realm::new().unwrap();
        
        // This should not fail - demonstrates the setup pattern
        let result = ExampleBuiltinType::setup_in_realm(&mut realm);
        assert!(result.is_ok());
    }
}
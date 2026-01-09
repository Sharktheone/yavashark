// 7.4.2 GetIteratorDirect ( obj )
//
// Internal iterator record - stores iterator + cached next method

use crate::{ObjectHandle, Realm, Res, Value};

/// Internal iterator record - stores iterator + cached next method
/// 7.4.2 GetIteratorDirect ( obj )
/// This is a non-GC'd wrapper that holds ObjectHandles
#[derive(Debug, Clone)]
pub struct IteratorRecord {
    /// The iterator object
    pub iterator: ObjectHandle,
    /// Cached next method (avoids repeated property lookup)
    pub next_method: ObjectHandle,
}

impl IteratorRecord {
    /// 7.4.2 GetIteratorDirect ( obj )
    pub fn new(iterator: ObjectHandle, realm: &mut Realm) -> Res<Self> {
        // 1. Let nextMethod be ? Get(obj, "next").
        let next_method = iterator.get("next", realm)?.to_object()?;
        // 2. Let iteratorRecord be the Iterator Record { [[Iterator]]: obj, [[NextMethod]]: nextMethod, [[Done]]: false }.
        Ok(Self {
            iterator,
            next_method,
        })
    }

    /// Create from pre-fetched parts (used by WrapForValidIteratorPrototype)
    pub fn from_parts(iterator: ObjectHandle, next_method: ObjectHandle) -> Self {
        Self {
            iterator,
            next_method,
        }
    }

    /// 7.4.6 IteratorStep ( iteratorRecord ) + 7.4.7 IteratorStepValue ( iteratorRecord )
    /// Returns Some(value) for next value, None if done
    pub fn step(&self, realm: &mut Realm) -> Res<Option<Value>> {
        // 1. Let result be ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).
        let result = self
            .next_method
            .call(vec![], self.iterator.clone().into(), realm)?;

        // 2. If Type(result) is not Object, throw a TypeError exception.
        let result_obj = result.to_object()?;

        // 3. Let done be ? IteratorComplete(result).
        let done = result_obj.get("done", realm)?.is_truthy();

        // 4. If done is true, return done.
        if done {
            return Ok(None);
        }

        // 5. Let value be ? Get(result, "value").
        let value = result_obj.get("value", realm)?;

        // 6. Return value.
        Ok(Some(value))
    }

    /// 7.4.8 IteratorClose ( iteratorRecord, completion )
    pub fn close(&self, realm: &mut Realm) -> Res<()> {
        // 1. Assert: iteratorRecord.[[Iterator]] is an Object.
        // 2. Let iterator be iteratorRecord.[[Iterator]].
        // 3. Let innerResult be Completion(GetMethod(iterator, "return")).
        let return_method = self.iterator.get("return", realm)?;

        // 4. If innerResult.[[Value]] is not undefined, then
        if return_method.is_callable() {
            // a. Set innerResult to Completion(Call(innerResult.[[Value]], iterator)).
            return_method.call(realm, vec![], self.iterator.clone().into())?;
        }

        Ok(())
    }
}

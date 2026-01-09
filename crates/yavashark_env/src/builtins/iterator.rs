// 27.1 Iterator Objects
//
// This module implements the Iterator constructor and prototype methods
// according to ECMAScript 2026 specification.

mod helper;
mod iterator_helper_obj;
mod iterator_prototype;
mod record;

pub use helper::*;
pub use iterator_prototype::*;
pub use record::*;

use iterator_helper_obj::IteratorHelperObject;
use crate::native_obj::NativeObject;
use crate::realm::Intrinsic;
use crate::value::IntoValue;
use crate::value::Obj;
use crate::{Error, NativeFunction, Object, ObjectHandle, PropertyKey, Realm, Res, Symbol, Value, Variable};
use yavashark_macro::props;

/// %Iterator% - The Iterator constructor (27.1.3)
///
/// Iterator is an abstract class designed to be subclassed.
/// Calling `new Iterator()` directly throws TypeError unless
/// NewTarget is a subclass.
pub struct Iterator;

// 27.1.3 The Iterator Constructor
#[props(intrinsic_name = iterator, extends = IteratorPrototype, constructor_name = "Iterator")]
impl Iterator {
    /// 27.1.3.1 Iterator ( )
    /// The Iterator constructor is designed to be subclassable.
    /// Note: Without proper NewTarget support, we can't perfectly implement the spec.
    /// Per spec, `new Iterator()` should throw if NewTarget is Iterator itself.
    /// But subclass calls like `new SubIterator()` should work.
    /// 
    /// Current behavior: We allow construction to support subclassing.
    /// This means `new Iterator()` won't throw (deviates from spec).
    #[constructor]
    #[length(0)]
    fn construct(#[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // Per spec, we should check:
        // 1. If NewTarget is undefined, throw TypeError
        // 2. If NewTarget is Iterator itself, throw TypeError
        // 3. Otherwise, return OrdinaryCreateFromConstructor(NewTarget, "%Iterator.prototype%")
        //
        // Since we can't check NewTarget, we return an object with Iterator.prototype
        // to support subclassing. This allows `class Sub extends Iterator {}; new Sub()` to work.
        Ok(crate::Object::raw_with_proto(Self::get_intrinsic(realm)?).into_object())
    }

    // === Static methods (on Iterator constructor) ===

    /// 27.1.3.2 Iterator.from ( O )
    #[prop("from")]
    fn from(o: Value, #[realm] realm: &mut Realm) -> Res<Value> {
        // 1. If O is a String, set O to ! ToObject(O).
        let o = if o.is_string() {
            o.to_object()?.into()
        } else {
            o
        };

        // 2. If O is not an Object, throw TypeError
        if !o.is_object() {
            return Err(Error::ty("Iterator.from requires an object or string"));
        }
        let o_obj = o.clone().to_object()?;

        // 3. Let iteratorRecord be ? GetIteratorFlattenable(O, iterate-strings).
        // GetIteratorFlattenable checks for [Symbol.iterator] first
        let iterator_method = o_obj.get(Symbol::ITERATOR, realm)?;

        if iterator_method.is_callable() {
            let iter = iterator_method.call(realm, vec![], o)?;
            let iterator_obj = iter.clone().to_object()?;

            // 4. Let hasInstance be ? OrdinaryHasInstance(%Iterator%, iteratorRecord.[[Iterator]]).
            // 5. If hasInstance is true, return iteratorRecord.[[Iterator]].
            let iterator_proto = Self::get_intrinsic(realm)?;
            if has_in_prototype_chain(&iterator_obj, &iterator_proto, realm)? {
                return Ok(iter);
            }

            // 6. Let wrapper be OrdinaryObjectCreate(%WrapForValidIteratorPrototype%, « [[Iterated]] »).
            // 7. Set wrapper.[[Iterated]] to iteratorRecord.
            let next_method = iterator_obj.get("next", realm)?.to_object()?;
            let wrapped = WrapForValidIteratorPrototype::new(iterator_obj, next_method, realm)?;
            return Ok(wrapped.into_object().into());
        }

        // No [Symbol.iterator], check for "next" method (iterator-like)
        let next_method = o_obj.get("next", realm)?;
        if !next_method.is_callable() {
            return Err(Error::ty("Object is not iterable"));
        }

        let wrapped =
            WrapForValidIteratorPrototype::new(o_obj, next_method.to_object()?, realm)?;
        Ok(wrapped.into_object().into())
    }

    /// 27.1.3.3 Iterator.concat ( ...items )
    #[prop("concat")]
    #[length(0)]
    fn concat(args: Vec<Value>, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let iterables be a new empty List.
        let mut iterables = Vec::new();

        // 2. For each element item of items, do
        for item in args {
            // a. If item is not an Object, throw a TypeError exception.
            let item_obj = item.clone().to_object()?;

            // b. Let method be ? GetMethod(item, @@iterator).
            let method = item_obj.get(Symbol::ITERATOR, realm)?;

            // c. If method is undefined, throw a TypeError exception.
            if !method.is_callable() {
                return Err(Error::ty("Argument is not iterable"));
            }

            // d. Append the Record { [[OpenMethod]]: method, [[Iterable]]: item } to iterables.
            // Cache the method so we only get it once
            iterables.push((method.to_object()?, item));
        }

        // 3. Let closure be a new Abstract Closure with no parameters that captures iterables
        // 4. Return CreateIteratorFromClosure(closure, "Iterator Helper", %IteratorHelperPrototype%)
        ConcatIteratorHelper::create(iterables, realm)
    }

    /// 27.1.3.4 Iterator.zip ( iterables [ , options ] )
    #[prop("zip")]
    fn zip(args: Vec<Value>, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let iterables be ? GetIteratorFlattenable(iterablesArg, iterate-strings).
        let iterables_arg = args.first().cloned().unwrap_or(Value::Undefined);
        let iterables_obj = iterables_arg.clone().to_object()?;

        // Get iterator from iterables argument
        let iterator_method = iterables_obj.get(Symbol::ITERATOR, realm)?;
        if !iterator_method.is_callable() {
            return Err(Error::ty("First argument is not iterable"));
        }

        let iterables_iter = iterator_method.call(realm, vec![], iterables_arg)?;
        let iterables_iter_obj = iterables_iter.to_object()?;
        let iterables_record = IteratorRecord::new(iterables_iter_obj, realm)?;

        // Collect all iterables into a list
        let mut iterable_list = Vec::new();
        loop {
            match iterables_record.step(realm)? {
                Some(v) => iterable_list.push(v),
                None => break,
            }
        }

        // 2. Let options be ? GetOptionsObject(options).
        let options_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
        let (mode, padding) = parse_zip_options(&options_arg, iterable_list.len(), realm)?;

        // 3. For each element value of iterables, get its iterator
        let mut iterator_records: Vec<IteratorRecord> = Vec::new();
        for iterable in &iterable_list {
            let iterable_obj = iterable.clone().to_object()?;
            let iter_method = iterable_obj.get(Symbol::ITERATOR, realm)?;
            if !iter_method.is_callable() {
                // Close all previously opened iterators
                for rec in &iterator_records {
                    let _ = rec.close(realm);
                }
                return Err(Error::ty("Element is not iterable"));
            }
            let iter = iter_method.call(realm, vec![], iterable.clone())?;
            let iter_obj = iter.to_object()?;
            let record = IteratorRecord::new(iter_obj, realm)?;
            iterator_records.push(record);
        }

        ZipIteratorHelper::create(iterator_records, mode, padding, realm)
    }

    /// 27.1.3.5 Iterator.zipKeyed ( iterables [ , options ] )
    #[prop("zipKeyed")]
    fn zip_keyed(args: Vec<Value>, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. If iterables is not an Object, throw a TypeError exception.
        let iterables_arg = args.first().cloned().unwrap_or(Value::Undefined);
        let iterables_obj = iterables_arg.to_object()?;

        // 2. Get the keys of the iterables object
        let keys = iterables_obj.keys(realm)?;

        // 3. Let options be ? GetOptionsObject(options).
        let options_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
        let (mode, padding_values) = parse_zip_keyed_options(&options_arg, &keys, &iterables_obj, realm)?;

        // 4. For each key, get the iterator
        let mut iterator_records: Vec<(String, IteratorRecord)> = Vec::new();
        for key in &keys {
            let key_str = key.as_str().to_string();
            let iterable = iterables_obj.get(key.clone(), realm)?;
            let iterable_obj = iterable.clone().to_object()?;
            let iter_method = iterable_obj.get(Symbol::ITERATOR, realm)?;
            if !iter_method.is_callable() {
                // Close all previously opened iterators
                for (_, rec) in &iterator_records {
                    let _ = rec.close(realm);
                }
                return Err(Error::ty("Property is not iterable"));
            }
            let iter = iter_method.call(realm, vec![], iterable)?;
            let iter_obj = iter.to_object()?;
            let record = IteratorRecord::new(iter_obj, realm)?;
            iterator_records.push((key_str, record));
        }

        ZipKeyedIteratorHelper::create(iterator_records, mode, padding_values, realm)
    }

    // === Prototype methods ===

    /// @@toStringTag getter
    #[get(Symbol::TO_STRING_TAG)]
    #[nonstatic]
    fn to_string_tag() -> &'static str {
        "Iterator"
    }

    /// 27.1.2.3 Iterator.prototype.drop ( limit )
    #[nonstatic]
    fn drop(#[this] this: Value, limit: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 3. Let numLimit be ? ToNumber(limit).
        let num_limit = limit.to_number(realm)?;

        // 4. If numLimit is NaN, throw a RangeError exception.
        if num_limit.is_nan() {
            return Err(Error::range("limit must not be NaN"));
        }

        // 5. Let integerLimit be ! ToIntegerOrInfinity(numLimit).
        // 6. If integerLimit < 0, throw a RangeError exception.
        let integer_limit = to_integer_or_infinity(num_limit);
        if integer_limit < 0.0 {
            return Err(Error::range("limit must not be negative"));
        }

        // 7. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        // 8-9. Return CreateIteratorFromClosure with drop logic
        DropIteratorHelper::create(iterated, integer_limit as u64, realm)
    }

    /// 27.1.2.4 Iterator.prototype.every ( predicate )
    #[nonstatic]
    fn every(#[this] this: Value, predicate: Value, #[realm] realm: &mut Realm) -> Res<bool> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 2. If IsCallable(predicate) is false, throw a TypeError exception.
        if !predicate.is_callable() {
            return Err(Error::ty("predicate is not callable"));
        }
        let predicate = predicate.to_object()?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        // 4. Let counter be 0.
        let mut counter = 0u64;

        // 5. Repeat
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let value = match iterated.step(realm)? {
                Some(v) => v,
                None => {
                    // b. If value is done, return true.
                    return Ok(true);
                }
            };

            // c. Let result be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).
            let result = match predicate.call(
                vec![value, Value::Number(counter as f64)],
                Value::Undefined,
                realm,
            ) {
                Ok(v) => v,
                Err(e) => {
                    // d. IfAbruptCloseIterator(result, iterated).
                    let _ = iterated.close(realm);
                    return Err(e);
                }
            };

            // e. If ToBoolean(result) is false, return ? IteratorClose(iterated, NormalCompletion(false)).
            if !result.is_truthy() {
                let _ = iterated.close(realm);
                return Ok(false);
            }

            // f. Set counter to counter + 1.
            counter += 1;
        }
    }

    /// 27.1.2.5 Iterator.prototype.filter ( predicate )
    #[nonstatic]
    fn filter(#[this] this: Value, predicate: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 2. If IsCallable(predicate) is false, throw a TypeError exception.
        if !predicate.is_callable() {
            return Err(Error::ty("predicate is not a function"));
        }
        let predicate = predicate.to_object()?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        FilterIteratorHelper::create(iterated, predicate, realm)
    }

    /// 27.1.2.6 Iterator.prototype.find ( predicate )
    #[nonstatic]
    fn find(#[this] this: Value, predicate: Value, #[realm] realm: &mut Realm) -> Res<Value> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 2. If IsCallable(predicate) is false, throw a TypeError exception.
        if !predicate.is_callable() {
            return Err(Error::ty("predicate is not callable"));
        }
        let predicate = predicate.to_object()?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        // 4. Let counter be 0.
        let mut counter = 0u64;

        // 5. Repeat
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let value = match iterated.step(realm)? {
                Some(v) => v,
                None => {
                    // b. If value is done, return undefined.
                    return Ok(Value::Undefined);
                }
            };

            // c. Let result be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).
            let result = match predicate.call(
                vec![value.clone(), Value::Number(counter as f64)],
                Value::Undefined,
                realm,
            ) {
                Ok(v) => v,
                Err(e) => {
                    // d. IfAbruptCloseIterator(result, iterated).
                    let _ = iterated.close(realm);
                    return Err(e);
                }
            };

            // e. If ToBoolean(result) is true, return ? IteratorClose(iterated, NormalCompletion(value)).
            if result.is_truthy() {
                let _ = iterated.close(realm);
                return Ok(value);
            }

            // f. Set counter to counter + 1.
            counter += 1;
        }
    }

    /// 27.1.2.7 Iterator.prototype.flatMap ( mapper )
    #[nonstatic]
    #[prop("flatMap")]
    fn flat_map(#[this] this: Value, mapper: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 2. If IsCallable(mapper) is false, throw a TypeError exception.
        if !mapper.is_callable() {
            return Err(Error::ty("mapper is not a function"));
        }
        let mapper = mapper.to_object()?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        FlatMapIteratorHelper::create(iterated, mapper, realm)
    }

    /// 27.1.2.8 Iterator.prototype.forEach ( procedure )
    #[nonstatic]
    #[prop("forEach")]
    fn for_each(#[this] this: Value, procedure: Value, #[realm] realm: &mut Realm) -> Res<()> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 2. If IsCallable(procedure) is false, throw a TypeError exception.
        if !procedure.is_callable() {
            return Err(Error::ty("procedure is not callable"));
        }
        let procedure = procedure.to_object()?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        // 4. Let counter be 0.
        let mut counter = 0u64;

        // 5. Repeat
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let value = match iterated.step(realm)? {
                Some(v) => v,
                None => {
                    // b. If value is done, return undefined.
                    return Ok(());
                }
            };

            // c. Let result be Completion(Call(procedure, undefined, « value, 𝔽(counter) »)).
            if let Err(e) = procedure.call(
                vec![value, Value::Number(counter as f64)],
                Value::Undefined,
                realm,
            ) {
                // d. IfAbruptCloseIterator(result, iterated).
                let _ = iterated.close(realm);
                return Err(e);
            }

            // e. Set counter to counter + 1.
            counter += 1;
        }
    }

    /// 27.1.2.9 Iterator.prototype.map ( mapper )
    #[nonstatic]
    fn map(#[this] this: Value, mapper: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 2. If IsCallable(mapper) is false, throw a TypeError exception.
        if !mapper.is_callable() {
            return Err(Error::ty("mapper is not a function"));
        }
        let mapper = mapper.to_object()?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        MapIteratorHelper::create(iterated, mapper, realm)
    }

    /// 27.1.2.10 Iterator.prototype.reduce ( reducer [ , initialValue ] )
    #[nonstatic]
    fn reduce(#[this] this: Value, reducer: Value, initial_value: Option<Value>, #[realm] realm: &mut Realm) -> Res<Value> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 2. If IsCallable(reducer) is false, throw a TypeError exception.
        if !reducer.is_callable() {
            return Err(Error::ty("reducer is not callable"));
        }
        let reducer = reducer.to_object()?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        // 4. If initialValue is not present
        let mut accumulator = if let Some(init) = initial_value {
            init
        } else {
            // a. Let value be ? IteratorStepValue(iterated).
            // b. If value is done, throw a TypeError exception.
            match iterated.step(realm)? {
                Some(v) => v,
                None => {
                    return Err(Error::ty("Reduce of empty iterator with no initial value"));
                }
            }
        };

        // 5. Let counter be 0.
        let mut counter = 0u64;

        // 6. Repeat
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let value = match iterated.step(realm)? {
                Some(v) => v,
                None => {
                    // b. If value is done, return accumulator.
                    return Ok(accumulator);
                }
            };

            // c. Let result be Completion(Call(reducer, undefined, « accumulator, value, 𝔽(counter) »)).
            accumulator = match reducer.call(
                vec![accumulator, value, Value::Number(counter as f64)],
                Value::Undefined,
                realm,
            ) {
                Ok(v) => v,
                Err(e) => {
                    // d. IfAbruptCloseIterator(result, iterated).
                    let _ = iterated.close(realm);
                    return Err(e);
                }
            };

            // e. Set counter to counter + 1.
            counter += 1;
        }
    }

    /// 27.1.2.11 Iterator.prototype.some ( predicate )
    #[nonstatic]
    fn some(#[this] this: Value, predicate: Value, #[realm] realm: &mut Realm) -> Res<bool> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 2. If IsCallable(predicate) is false, throw a TypeError exception.
        if !predicate.is_callable() {
            return Err(Error::ty("predicate is not callable"));
        }
        let predicate = predicate.to_object()?;

        // 3. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        // 4. Let counter be 0.
        let mut counter = 0u64;

        // 5. Repeat
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            let value = match iterated.step(realm)? {
                Some(v) => v,
                None => {
                    // b. If value is done, return false.
                    return Ok(false);
                }
            };

            // c. Let result be Completion(Call(predicate, undefined, « value, 𝔽(counter) »)).
            let result = match predicate.call(
                vec![value, Value::Number(counter as f64)],
                Value::Undefined,
                realm,
            ) {
                Ok(v) => v,
                Err(e) => {
                    // d. IfAbruptCloseIterator(result, iterated).
                    let _ = iterated.close(realm);
                    return Err(e);
                }
            };

            // e. If ToBoolean(result) is true, return ? IteratorClose(iterated, NormalCompletion(true)).
            if result.is_truthy() {
                let _ = iterated.close(realm);
                return Ok(true);
            }

            // f. Set counter to counter + 1.
            counter += 1;
        }
    }

    /// 27.1.2.12 Iterator.prototype.take ( limit )
    #[nonstatic]
    fn take(#[this] this: Value, limit: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 3. Let numLimit be ? ToNumber(limit).
        let num_limit = limit.to_number(realm)?;

        // 4. If numLimit is NaN, throw a RangeError exception.
        if num_limit.is_nan() {
            return Err(Error::range("limit must not be NaN"));
        }

        // 5. Let integerLimit be ! ToIntegerOrInfinity(numLimit).
        // 6. If integerLimit < 0, throw a RangeError exception.
        let integer_limit = to_integer_or_infinity(num_limit);
        if integer_limit < 0.0 {
            return Err(Error::range("limit must not be negative"));
        }

        // 7. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        // 8-9. Return CreateIteratorFromClosure with take logic
        TakeIteratorHelper::create(iterated, integer_limit as u64, realm)
    }

    /// 27.1.2.13 Iterator.prototype.toArray ( )
    #[nonstatic]
    #[prop("toArray")]
    fn to_array(#[this] this: Value, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let O be the this value.
        let o = this.to_object()?;

        // 2. Let iterated be ? GetIteratorDirect(O).
        let iterated = IteratorRecord::new(o, realm)?;

        // 3. Let items be a new empty List.
        let mut items = Vec::new();

        // 4. Repeat
        loop {
            // a. Let value be ? IteratorStepValue(iterated).
            match iterated.step(realm)? {
                Some(v) => {
                    // c. Append value to items.
                    items.push(v);
                }
                None => {
                    // b. If value is done, return CreateArrayFromList(items).
                    return crate::array::Array::with_elements(realm, items)
                        .map(|a| a.into_object());
                }
            }
        }
    }

    // Note: @@dispose is inherited from %IteratorPrototype%
}

/// Parse options for Iterator.zip
fn parse_zip_options(
    options: &Value,
    _count: usize,
    realm: &mut Realm,
) -> Res<(ZipMode, Vec<Value>)> {
    if options.is_undefined() || options.is_null() {
        return Ok((ZipMode::Shortest, Vec::new()));
    }

    let options_obj = options.clone().to_object()?;

    // Get mode option
    let mode_val = options_obj.get("mode", realm)?;
    let mode = if mode_val.is_undefined() {
        ZipMode::Shortest
    } else {
        let mode_str = mode_val.to_string(realm)?;
        match mode_str.as_str() {
            "shortest" => ZipMode::Shortest,
            "longest" => ZipMode::Longest,
            "strict" => ZipMode::Strict,
            _ => return Err(Error::range("Invalid mode")),
        }
    };

    // Get padding option (only used in longest mode)
    let padding = if mode == ZipMode::Longest {
        let padding_val = options_obj.get("padding", realm)?;
        if !padding_val.is_undefined() {
            // padding should be an iterable
            let padding_obj = padding_val.clone().to_object()?;
            let iter_method = padding_obj.get(Symbol::ITERATOR, realm)?;
            if iter_method.is_callable() {
                let iter = iter_method.call(realm, vec![], padding_val)?;
                let iter_obj = iter.to_object()?;
                let record = IteratorRecord::new(iter_obj, realm)?;
                let mut padding_list = Vec::new();
                loop {
                    match record.step(realm)? {
                        Some(v) => padding_list.push(v),
                        None => break,
                    }
                }
                padding_list
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok((mode, padding))
}

/// Parse options for Iterator.zipKeyed
fn parse_zip_keyed_options(
    options: &Value,
    keys: &[PropertyKey],
    _iterables: &ObjectHandle,
    realm: &mut Realm,
) -> Res<(ZipMode, Vec<(String, Value)>)> {
    if options.is_undefined() || options.is_null() {
        return Ok((ZipMode::Shortest, Vec::new()));
    }

    let options_obj = options.clone().to_object()?;

    // Get mode option
    let mode_val = options_obj.get("mode", realm)?;
    let mode = if mode_val.is_undefined() {
        ZipMode::Shortest
    } else {
        let mode_str = mode_val.to_string(realm)?;
        match mode_str.as_str() {
            "shortest" => ZipMode::Shortest,
            "longest" => ZipMode::Longest,
            "strict" => ZipMode::Strict,
            _ => return Err(Error::range("Invalid mode")),
        }
    };

    // Get padding option (only used in longest mode)
    let padding = if mode == ZipMode::Longest {
        let padding_val = options_obj.get("padding", realm)?;
        if !padding_val.is_undefined() {
            // padding should be an object with the same keys
            let padding_obj = padding_val.to_object()?;
            let mut padding_list = Vec::new();
            for key in keys {
                let key_str = key.as_str().to_string();
                let val = padding_obj.get(key.clone(), realm)?;
                padding_list.push((key_str, val));
            }
            padding_list
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok((mode, padding))
}

/// 7.1.5 ToIntegerOrInfinity ( argument )
fn to_integer_or_infinity(num: f64) -> f64 {
    if num.is_nan() || num == 0.0 {
        0.0
    } else if num.is_infinite() {
        num
    } else {
        num.trunc()
    }
}

/// Check if obj has proto in its prototype chain
fn has_in_prototype_chain(obj: &ObjectHandle, proto: &ObjectHandle, realm: &mut Realm) -> Res<bool> {
    use crate::ObjectOrNull;
    let mut current = obj.prototype(realm)?;
    loop {
        match current {
            ObjectOrNull::Object(p) => {
                if p.eq(proto) {
                    return Ok(true);
                }
                current = p.prototype(realm)?;
            }
            ObjectOrNull::Null => return Ok(false),
        }
    }
}

// ============================================================================
// WrapForValidIteratorPrototype
// ============================================================================

/// %WrapForValidIteratorPrototype% - Wraps iterator-like objects
/// 27.1.4.1 The %WrapForValidIteratorPrototype% Object
/// Used by Iterator.from when the object isn't already an Iterator
#[derive(Debug)]
pub struct WrapForValidIteratorPrototype {
    /// The underlying iterator record
    iterated: IteratorRecord,
}

impl WrapForValidIteratorPrototype {
    pub fn new(
        iterator: ObjectHandle,
        next_method: ObjectHandle,
        realm: &mut Realm,
    ) -> Res<NativeObject<Self>> {
        NativeObject::new(
            Self {
                iterated: IteratorRecord::from_parts(iterator, next_method),
            },
            realm,
        )
    }
    
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let O be this value.
        // 2. Let iteratorRecord be O.[[Iterated]].
        // 3. Return ? Call(iteratorRecord.[[NextMethod]], iteratorRecord.[[Iterator]]).
        let result = self.iterated.next_method.call(
            vec![],
            self.iterated.iterator.clone().into(),
            realm,
        )?;
        result.to_object()
    }

    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // 1. Let O be this value.
        // 2. Let iterator be O.[[Iterated]].[[Iterator]].
        // 3. Let returnMethod be ? GetMethod(iterator, "return").
        let return_method = self.iterated.iterator.get("return", realm)?;

        // 4. If returnMethod is undefined, return CreateIteratorResultObject(undefined, true).
        if !return_method.is_callable() {
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        // 5. Return ? Call(returnMethod, iterator).
        let result = return_method.call(realm, vec![], self.iterated.iterator.clone().into())?;
        result.to_object()
    }
}

/// Static intrinsic for WrapForValidIteratorPrototype
pub struct WrapForValidIteratorPrototypeIntrinsic;

impl WrapForValidIteratorPrototypeIntrinsic {
    pub fn init(realm: &mut Realm) -> Res<ObjectHandle> {
        let iterator_proto = Iterator::get_intrinsic(realm)?;
        let proto = Object::raw_with_proto(iterator_proto);

        // Define next method that dispatches to the wrapped iterator
        let next_fn = NativeFunction::with_proto_and_len(
            "next",
            |_args, this, realm| {
                let this_obj = this.to_object()?;
                let helper = this_obj
                    .downcast::<NativeObject<WrapForValidIteratorPrototype>>()
                    .ok_or_else(|| Error::ty("not a wrapped iterator"))?;
                helper.next_impl(realm).map(|o| o.into())
            },
            realm.intrinsics.func.clone(),
            0,
            realm,
        );
        proto.define_property_attributes(
            "next".into(),
            Variable::write_config(next_fn.into()),
            realm,
        )?;

        // Define return method
        let return_fn = NativeFunction::with_proto_and_len(
            "return",
            |_args, this, realm| {
                let this_obj = this.to_object()?;
                let helper = this_obj
                    .downcast::<NativeObject<WrapForValidIteratorPrototype>>()
                    .ok_or_else(|| Error::ty("not a wrapped iterator"))?;
                helper.return_impl(realm).map(|o| o.into())
            },
            realm.intrinsics.func.clone(),
            0,
            realm,
        );
        proto.define_property_attributes(
            "return".into(),
            Variable::write_config(return_fn.into()),
            realm,
        )?;

        Ok(proto.into_object())
    }
}

impl Intrinsic for WrapForValidIteratorPrototype {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        WrapForValidIteratorPrototypeIntrinsic::init(realm)
    }

    fn get_intrinsic(realm: &mut Realm) -> Res<ObjectHandle> {
        // Store this in the "other" intrinsics map
        if let Ok(proto) = realm.intrinsics.get_of::<WrapForValidIteratorPrototype>() {
            return Ok(proto);
        }
        let proto = Self::initialize(realm)?;
        realm.intrinsics.insert::<WrapForValidIteratorPrototype>(proto.clone());
        Ok(proto)
    }

    fn get_global(realm: &mut Realm) -> Res<ObjectHandle> {
        Self::get_intrinsic(realm)
    }
}

// ============================================================================
// Iterator Helper Types
// ============================================================================

/// Macro to implement Intrinsic for iterator helper types.
/// All iterator helpers delegate to IteratorHelperPrototype for their intrinsic.
macro_rules! impl_iterator_helper_intrinsic {
    ($($helper:ty),+ $(,)?) => {
        $(
            impl Intrinsic for $helper {
                fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
                    IteratorHelperPrototype::initialize(realm)
                }

                fn get_intrinsic(realm: &mut Realm) -> Res<ObjectHandle> {
                    IteratorHelperPrototype::get_intrinsic(realm)
                }

                fn get_global(realm: &mut Realm) -> Res<ObjectHandle> {
                    IteratorHelperPrototype::get_global(realm)
                }
            }
        )+
    };
}

// Apply the macro to all iterator helper types
impl_iterator_helper_intrinsic!(
    MapIteratorHelper,
    FilterIteratorHelper,
    TakeIteratorHelper,
    DropIteratorHelper,
    FlatMapIteratorHelper,
    ConcatIteratorHelper,
    ZipIteratorHelper,
    ZipKeyedIteratorHelper,
);

/// MapIteratorHelper - Iterator helper for map() operation
/// 27.1.2.9 Iterator.prototype.map ( mapper )
#[derive(Debug)]
pub struct MapIteratorHelper {
    /// The underlying iterator record
    iterated: IteratorRecord,
    /// The mapper function
    mapper: ObjectHandle,
    /// Counter for mapper's second argument
    counter: std::cell::Cell<u64>,
    /// Whether the iterator is alive (not done)
    alive: std::cell::Cell<bool>,
}

impl MapIteratorHelper {
    pub fn create(
        iterated: IteratorRecord,
        mapper: ObjectHandle,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let helper = IteratorHelperObject::new(
            Self {
                iterated,
                mapper,
                counter: std::cell::Cell::new(0),
                alive: std::cell::Cell::new(true),
            },
            realm,
        )?;
        Ok(helper.into_object())
    }
}

impl IteratorHelperImpl for MapIteratorHelper {
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        if !self.alive.get() {
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        // Get next value from underlying iterator
        let value = match self.iterated.step(realm) {
            Ok(Some(v)) => v,
            Ok(None) => {
                self.alive.set(false);
                return create_iter_result_object(Value::Undefined, true, realm);
            }
            Err(e) => {
                self.alive.set(false);
                return Err(e);
            }
        };

        // Call mapper with (value, counter)
        let counter = self.counter.get();
        self.counter.set(counter + 1);

        let mapped = match self.mapper.call(
            vec![value, Value::Number(counter as f64)],
            Value::Undefined,
            realm,
        ) {
            Ok(v) => v,
            Err(e) => {
                self.alive.set(false);
                let _ = self.iterated.close(realm);
                return Err(e);
            }
        };

        create_iter_result_object(mapped, false, realm)
    }

    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // Only close the underlying iterator if we haven't already
        if self.alive.get() {
            self.alive.set(false);
            self.iterated.close(realm)?;
        }
        create_iter_result_object(Value::Undefined, true, realm)
    }
}

/// FilterIteratorHelper - Iterator helper for filter() operation
/// 27.1.2.5 Iterator.prototype.filter ( predicate )
#[derive(Debug)]
pub struct FilterIteratorHelper {
    /// The underlying iterator record
    iterated: IteratorRecord,
    /// The predicate function
    predicate: ObjectHandle,
    /// Counter for predicate's second argument
    counter: std::cell::Cell<u64>,
    /// Whether the iterator is alive
    alive: std::cell::Cell<bool>,
}

impl FilterIteratorHelper {
    pub fn create(
        iterated: IteratorRecord,
        predicate: ObjectHandle,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let helper = IteratorHelperObject::new(
            Self {
                iterated,
                predicate,
                counter: std::cell::Cell::new(0),
                alive: std::cell::Cell::new(true),
            },
            realm,
        )?;
        Ok(helper.into_object())
    }
}

impl IteratorHelperImpl for FilterIteratorHelper {
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        if !self.alive.get() {
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        // Keep trying until we find a value that passes the predicate
        loop {
            let value = match self.iterated.step(realm) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    self.alive.set(false);
                    return create_iter_result_object(Value::Undefined, true, realm);
                }
                Err(e) => {
                    self.alive.set(false);
                    return Err(e);
                }
            };

            let counter = self.counter.get();
            self.counter.set(counter + 1);

            let selected = match self.predicate.call(
                vec![value.clone(), Value::Number(counter as f64)],
                Value::Undefined,
                realm,
            ) {
                Ok(v) => v,
                Err(e) => {
                    self.alive.set(false);
                    let _ = self.iterated.close(realm);
                    return Err(e);
                }
            };

            if selected.is_truthy() {
                return create_iter_result_object(value, false, realm);
            }
        }
    }

    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // Only close the underlying iterator if we haven't already
        if self.alive.get() {
            self.alive.set(false);
            self.iterated.close(realm)?;
        }
        create_iter_result_object(Value::Undefined, true, realm)
    }
}

/// TakeIteratorHelper - Iterator helper for take() operation
/// 27.1.2.12 Iterator.prototype.take ( limit )
#[derive(Debug)]
pub struct TakeIteratorHelper {
    /// The underlying iterator record
    iterated: IteratorRecord,
    /// Remaining items to take
    remaining: std::cell::Cell<u64>,
    /// Whether the iterator is alive
    alive: std::cell::Cell<bool>,
}

impl TakeIteratorHelper {
    pub fn create(
        iterated: IteratorRecord,
        limit: u64,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let helper = IteratorHelperObject::new(
            Self {
                iterated,
                remaining: std::cell::Cell::new(limit),
                alive: std::cell::Cell::new(true),
            },
            realm,
        )?;
        Ok(helper.into_object())
    }
}

impl IteratorHelperImpl for TakeIteratorHelper {
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        if !self.alive.get() {
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        let remaining = self.remaining.get();
        if remaining == 0 {
            self.alive.set(false);
            let _ = self.iterated.close(realm);
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        self.remaining.set(remaining - 1);

        match self.iterated.step(realm) {
            Ok(Some(v)) => create_iter_result_object(v, false, realm),
            Ok(None) => {
                self.alive.set(false);
                create_iter_result_object(Value::Undefined, true, realm)
            }
            Err(e) => {
                self.alive.set(false);
                Err(e)
            }
        }
    }

    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // Only close the underlying iterator if we haven't already
        if self.alive.get() {
            self.alive.set(false);
            self.iterated.close(realm)?;
        }
        create_iter_result_object(Value::Undefined, true, realm)
    }
}

/// DropIteratorHelper - Iterator helper for drop() operation
/// 27.1.2.3 Iterator.prototype.drop ( limit )
#[derive(Debug)]
pub struct DropIteratorHelper {
    /// The underlying iterator record
    iterated: IteratorRecord,
    /// Remaining items to skip
    remaining_to_skip: std::cell::Cell<u64>,
    /// Whether the iterator is alive
    alive: std::cell::Cell<bool>,
}

impl DropIteratorHelper {
    pub fn create(
        iterated: IteratorRecord,
        limit: u64,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let helper = IteratorHelperObject::new(
            Self {
                iterated,
                remaining_to_skip: std::cell::Cell::new(limit),
                alive: std::cell::Cell::new(true),
            },
            realm,
        )?;
        Ok(helper.into_object())
    }
}

impl IteratorHelperImpl for DropIteratorHelper {
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        if !self.alive.get() {
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        // Skip initial items if needed
        while self.remaining_to_skip.get() > 0 {
            match self.iterated.step(realm) {
                Ok(Some(_)) => {
                    self.remaining_to_skip
                        .set(self.remaining_to_skip.get() - 1);
                }
                Ok(None) => {
                    self.alive.set(false);
                    return create_iter_result_object(Value::Undefined, true, realm);
                }
                Err(e) => {
                    self.alive.set(false);
                    return Err(e);
                }
            }
        }

        // Now just pass through values
        match self.iterated.step(realm) {
            Ok(Some(v)) => create_iter_result_object(v, false, realm),
            Ok(None) => {
                self.alive.set(false);
                create_iter_result_object(Value::Undefined, true, realm)
            }
            Err(e) => {
                self.alive.set(false);
                Err(e)
            }
        }
    }

    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // Only close the underlying iterator if we haven't already
        if self.alive.get() {
            self.alive.set(false);
            self.iterated.close(realm)?;
        }
        create_iter_result_object(Value::Undefined, true, realm)
    }
}

/// FlatMapIteratorHelper - Iterator helper for flatMap() operation
/// 27.1.2.7 Iterator.prototype.flatMap ( mapper )
#[derive(Debug)]
pub struct FlatMapIteratorHelper {
    /// The underlying iterator record
    iterated: IteratorRecord,
    /// The mapper function
    mapper: ObjectHandle,
    /// Counter for mapper's second argument
    counter: std::cell::Cell<u64>,
    /// Current inner iterator (from mapped result)
    inner: std::cell::RefCell<Option<IteratorRecord>>,
    /// Whether the iterator is alive
    alive: std::cell::Cell<bool>,
}

impl FlatMapIteratorHelper {
    pub fn create(
        iterated: IteratorRecord,
        mapper: ObjectHandle,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let helper = IteratorHelperObject::new(
            Self {
                iterated,
                mapper,
                counter: std::cell::Cell::new(0),
                inner: std::cell::RefCell::new(None),
                alive: std::cell::Cell::new(true),
            },
            realm,
        )?;
        Ok(helper.into_object())
    }
}

impl IteratorHelperImpl for FlatMapIteratorHelper {
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        if !self.alive.get() {
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        loop {
            // If we have an inner iterator, try to get the next value from it
            {
                let inner_opt = self.inner.borrow();
                if let Some(ref inner) = *inner_opt {
                    match inner.step(realm) {
                        Ok(Some(v)) => {
                            return create_iter_result_object(v, false, realm);
                        }
                        Ok(None) => {
                            // Inner iterator exhausted, continue to get next outer value
                        }
                        Err(e) => {
                            drop(inner_opt);
                            self.alive.set(false);
                            let _ = self.iterated.close(realm);
                            return Err(e);
                        }
                    }
                }
            }

            // Clear the inner iterator
            *self.inner.borrow_mut() = None;

            // Get next value from outer iterator
            let value = match self.iterated.step(realm) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    self.alive.set(false);
                    return create_iter_result_object(Value::Undefined, true, realm);
                }
                Err(e) => {
                    self.alive.set(false);
                    return Err(e);
                }
            };

            let counter = self.counter.get();
            self.counter.set(counter + 1);

            // Call mapper to get an iterable
            let mapped = match self.mapper.call(
                vec![value, Value::Number(counter as f64)],
                Value::Undefined,
                realm,
            ) {
                Ok(v) => v,
                Err(e) => {
                    self.alive.set(false);
                    let _ = self.iterated.close(realm);
                    return Err(e);
                }
            };

            // Get iterator from mapped value
            let mapped_obj = match mapped.clone().to_object() {
                Ok(o) => o,
                Err(e) => {
                    self.alive.set(false);
                    let _ = self.iterated.close(realm);
                    return Err(e);
                }
            };

            // Try to get [Symbol.iterator] from mapped value
            let iterator_method = mapped_obj.get(Symbol::ITERATOR, realm)?;
            let inner_iterator = if iterator_method.is_callable() {
                let iter = iterator_method.call(realm, vec![], mapped)?;
                match iter.to_object() {
                    Ok(o) => o,
                    Err(e) => {
                        self.alive.set(false);
                        let _ = self.iterated.close(realm);
                        return Err(e);
                    }
                }
            } else {
                // If no @@iterator, treat mapped_obj as the iterator itself
                mapped_obj
            };

            // Create inner iterator record
            let inner_record = match IteratorRecord::new(inner_iterator, realm) {
                Ok(r) => r,
                Err(e) => {
                    self.alive.set(false);
                    let _ = self.iterated.close(realm);
                    return Err(e);
                }
            };

            *self.inner.borrow_mut() = Some(inner_record);
        }
    }

    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // Only close if we haven't already
        if self.alive.get() {
            self.alive.set(false);

            // Close inner iterator if present, collecting any error
            let inner_result = if let Some(ref inner) = *self.inner.borrow() {
                inner.close(realm)
            } else {
                Ok(())
            };

            // Close outer iterator, propagating any error
            let outer_result = self.iterated.close(realm);
            
            // Propagate errors (prefer outer error if both fail, per IteratorCloseAll semantics)
            inner_result?;
            outer_result?;
        }
        create_iter_result_object(Value::Undefined, true, realm)
    }
}

// ============================================================================
// ConcatIteratorHelper - Iterator.concat()
// ============================================================================

/// ConcatIteratorHelper - Iterator helper for Iterator.concat() operation
/// 27.1.3.3 Iterator.concat ( ...items )
///
/// Concatenates multiple iterables into a single iterator that yields
/// all values from each iterable in sequence.
#[derive(Debug)]
pub struct ConcatIteratorHelper {
    /// List of (method, iterable) pairs to concatenate
    iterables: std::cell::RefCell<Vec<(ObjectHandle, Value)>>,
    /// Current iterator being consumed
    current: std::cell::RefCell<Option<IteratorRecord>>,
    /// Index of next iterable to process
    index: std::cell::Cell<usize>,
    /// Whether the iterator is alive
    alive: std::cell::Cell<bool>,
}

impl ConcatIteratorHelper {
    /// 27.1.3.3 Iterator.concat ( ...items )
    /// Creates a new concat iterator from a list of (method, iterable) pairs
    pub fn create(iterables: Vec<(ObjectHandle, Value)>, realm: &mut Realm) -> Res<ObjectHandle> {
        let helper = IteratorHelperObject::new(
            Self {
                iterables: std::cell::RefCell::new(iterables),
                current: std::cell::RefCell::new(None),
                index: std::cell::Cell::new(0),
                alive: std::cell::Cell::new(true),
            },
            realm,
        )?;
        Ok(helper.into_object())
    }

    /// Get iterator from an iterable object using the cached method
    fn get_iterator(method: &ObjectHandle, iterable: &Value, realm: &mut Realm) -> Res<IteratorRecord> {
        let iter = method.call(vec![], iterable.clone(), realm)?;
        let iterator_obj = iter.to_object()?;
        IteratorRecord::new(iterator_obj, realm)
    }
}

impl IteratorHelperImpl for ConcatIteratorHelper {
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        if !self.alive.get() {
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        loop {
            // Try to get next value from current iterator
            {
                let current_opt = self.current.borrow();
                if let Some(ref current) = *current_opt {
                    match current.step(realm) {
                        Ok(Some(v)) => {
                            return create_iter_result_object(v, false, realm);
                        }
                        Ok(None) => {
                            // Current iterator exhausted, move to next
                        }
                        Err(e) => {
                            drop(current_opt);
                            self.alive.set(false);
                            return Err(e);
                        }
                    }
                }
            }

            // Clear current iterator
            *self.current.borrow_mut() = None;

            // Get next iterable
            let index = self.index.get();
            let iterables = self.iterables.borrow();
            
            if index >= iterables.len() {
                // All iterables exhausted
                self.alive.set(false);
                return create_iter_result_object(Value::Undefined, true, realm);
            }

            let (method, iterable) = iterables[index].clone();
            drop(iterables);
            self.index.set(index + 1);

            // Get iterator from the iterable using the cached method
            match Self::get_iterator(&method, &iterable, realm) {
                Ok(iter) => {
                    *self.current.borrow_mut() = Some(iter);
                }
                Err(e) => {
                    self.alive.set(false);
                    return Err(e);
                }
            }
        }
    }

    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // Only close if we haven't already
        if self.alive.get() {
            self.alive.set(false);

            // Close current iterator if present
            if let Some(ref current) = *self.current.borrow() {
                current.close(realm)?;
            }
        }

        create_iter_result_object(Value::Undefined, true, realm)
    }
}

// ============================================================================
// ZipIteratorHelper - Iterator.zip()
// ============================================================================

/// Mode for zip behavior when iterators have different lengths
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZipMode {
    /// Stop when the shortest iterator is exhausted (default)
    Shortest,
    /// Continue until the longest iterator is exhausted, filling with undefined
    Longest,
    /// Throw an error if iterators have different lengths
    Strict,
}

/// ZipIteratorHelper - Iterator helper for Iterator.zip() operation
/// 27.1.3.4 Iterator.zip ( iterables [ , options ] )
///
/// Combines multiple iterables into a single iterator that yields arrays
/// of values from each iterable.
#[derive(Debug)]
pub struct ZipIteratorHelper {
    /// Iterator records for all the iterables
    iterators: std::cell::RefCell<Vec<IteratorRecord>>,
    /// Zip mode (shortest, longest, or strict)
    mode: ZipMode,
    /// Padding value for "longest" mode
    padding: std::cell::RefCell<Vec<Value>>,
    /// Whether the iterator is alive
    alive: std::cell::Cell<bool>,
    /// Track which iterators are done (for longest mode)
    done: std::cell::RefCell<Vec<bool>>,
}

impl ZipIteratorHelper {
    /// Creates a new zip iterator from a list of iterator records
    pub fn create(
        iterators: Vec<IteratorRecord>,
        mode: ZipMode,
        padding: Vec<Value>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let len = iterators.len();
        let helper = IteratorHelperObject::new(
            Self {
                iterators: std::cell::RefCell::new(iterators),
                mode,
                padding: std::cell::RefCell::new(padding),
                alive: std::cell::Cell::new(true),
                done: std::cell::RefCell::new(vec![false; len]),
            },
            realm,
        )?;
        Ok(helper.into_object())
    }
}

impl IteratorHelperImpl for ZipIteratorHelper {
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        if !self.alive.get() {
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        let iterators = self.iterators.borrow();
        
        // If there are no iterators, we're immediately done
        if iterators.is_empty() {
            drop(iterators);
            self.alive.set(false);
            return create_iter_result_object(Value::Undefined, true, realm);
        }
        
        let mut done_flags = self.done.borrow_mut();
        let padding = self.padding.borrow();
        let mut values = Vec::with_capacity(iterators.len());
        let mut any_done = false;
        let mut all_done = true;

        for (i, iter) in iterators.iter().enumerate() {
            if done_flags[i] {
                // This iterator is already done
                any_done = true;
                match self.mode {
                    ZipMode::Longest => {
                        values.push(padding.get(i).cloned().unwrap_or(Value::Undefined));
                    }
                    _ => {
                        values.push(Value::Undefined);
                    }
                }
                continue;
            }

            match iter.step(realm) {
                Ok(Some(v)) => {
                    values.push(v);
                    all_done = false;
                }
                Ok(None) => {
                    done_flags[i] = true;
                    any_done = true;
                    match self.mode {
                        ZipMode::Longest => {
                            values.push(padding.get(i).cloned().unwrap_or(Value::Undefined));
                        }
                        _ => {
                            values.push(Value::Undefined);
                        }
                    }
                }
                Err(e) => {
                    drop(done_flags);
                    drop(iterators);
                    drop(padding);
                    self.alive.set(false);
                    // Close all iterators
                    for iter in self.iterators.borrow().iter() {
                        let _ = iter.close(realm);
                    }
                    return Err(e);
                }
            }
        }

        drop(done_flags);
        drop(iterators);
        drop(padding);

        match self.mode {
            ZipMode::Shortest => {
                if any_done {
                    self.alive.set(false);
                    // Close remaining iterators
                    for iter in self.iterators.borrow().iter() {
                        let _ = iter.close(realm);
                    }
                    return create_iter_result_object(Value::Undefined, true, realm);
                }
            }
            ZipMode::Longest => {
                if all_done {
                    self.alive.set(false);
                    return create_iter_result_object(Value::Undefined, true, realm);
                }
            }
            ZipMode::Strict => {
                let done_flags = self.done.borrow();
                let first_done = done_flags.first().copied().unwrap_or(false);
                if any_done && !done_flags.iter().all(|&d| d == first_done) {
                    drop(done_flags);
                    self.alive.set(false);
                    // Close all iterators
                    for iter in self.iterators.borrow().iter() {
                        let _ = iter.close(realm);
                    }
                    return Err(Error::ty(
                        "Iterators have different lengths in strict mode",
                    ));
                }
                if all_done {
                    self.alive.set(false);
                    return create_iter_result_object(Value::Undefined, true, realm);
                }
            }
        }

        // Create array from values
        let arr = crate::array::Array::with_elements(realm, values)?;
        create_iter_result_object(arr.into_value(), false, realm)
    }

    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // Only close if we haven't already
        if self.alive.get() {
            self.alive.set(false);

            // Close all iterators in reverse order per IteratorCloseAll
            let mut last_error: Option<crate::Error> = None;
            for iter in self.iterators.borrow().iter().rev() {
                if let Err(e) = iter.close(realm) {
                    last_error = Some(e);
                }
            }
            if let Some(e) = last_error {
                return Err(e);
            }
        }

        create_iter_result_object(Value::Undefined, true, realm)
    }
}

// ============================================================================
// ZipKeyedIteratorHelper - Iterator.zipKeyed()
// ============================================================================

/// ZipKeyedIteratorHelper - Iterator helper for Iterator.zipKeyed() operation
/// 27.1.3.5 Iterator.zipKeyed ( iterables [ , options ] )
///
/// Similar to zip, but takes an object with named iterables and yields
/// objects with the same keys mapped to values from each iterator.
#[derive(Debug)]
pub struct ZipKeyedIteratorHelper {
    /// Iterator records with their keys
    iterators: std::cell::RefCell<Vec<(String, IteratorRecord)>>,
    /// Zip mode (shortest, longest, or strict)
    mode: ZipMode,
    /// Padding values for "longest" mode, keyed by name
    padding: std::cell::RefCell<Vec<(String, Value)>>,
    /// Whether the iterator is alive
    alive: std::cell::Cell<bool>,
    /// Track which iterators are done (for longest mode)
    done: std::cell::RefCell<Vec<bool>>,
}

impl ZipKeyedIteratorHelper {
    /// Creates a new zipKeyed iterator
    pub fn create(
        iterators: Vec<(String, IteratorRecord)>,
        mode: ZipMode,
        padding: Vec<(String, Value)>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        let len = iterators.len();
        let helper = IteratorHelperObject::new(
            Self {
                iterators: std::cell::RefCell::new(iterators),
                mode,
                padding: std::cell::RefCell::new(padding),
                alive: std::cell::Cell::new(true),
                done: std::cell::RefCell::new(vec![false; len]),
            },
            realm,
        )?;
        Ok(helper.into_object())
    }
}

impl IteratorHelperImpl for ZipKeyedIteratorHelper {
    fn next_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        if !self.alive.get() {
            return create_iter_result_object(Value::Undefined, true, realm);
        }

        let iterators = self.iterators.borrow();
        
        // If there are no iterators, we're immediately done
        if iterators.is_empty() {
            drop(iterators);
            self.alive.set(false);
            return create_iter_result_object(Value::Undefined, true, realm);
        }
        
        let mut done_flags = self.done.borrow_mut();
        let padding = self.padding.borrow();
        let mut any_done = false;
        let mut all_done = true;
        let mut entries: Vec<(String, Value)> = Vec::with_capacity(iterators.len());

        for (i, (key, iter)) in iterators.iter().enumerate() {
            if done_flags[i] {
                any_done = true;
                match self.mode {
                    ZipMode::Longest => {
                        let pad_val = padding
                            .iter()
                            .find(|(k, _)| k == key)
                            .map(|(_, v)| v.clone())
                            .unwrap_or(Value::Undefined);
                        entries.push((key.clone(), pad_val));
                    }
                    _ => {
                        entries.push((key.clone(), Value::Undefined));
                    }
                }
                continue;
            }

            match iter.step(realm) {
                Ok(Some(v)) => {
                    entries.push((key.clone(), v));
                    all_done = false;
                }
                Ok(None) => {
                    done_flags[i] = true;
                    any_done = true;
                    match self.mode {
                        ZipMode::Longest => {
                            let pad_val = padding
                                .iter()
                                .find(|(k, _)| k == key)
                                .map(|(_, v)| v.clone())
                                .unwrap_or(Value::Undefined);
                            entries.push((key.clone(), pad_val));
                        }
                        _ => {
                            entries.push((key.clone(), Value::Undefined));
                        }
                    }
                }
                Err(e) => {
                    drop(done_flags);
                    drop(iterators);
                    drop(padding);
                    self.alive.set(false);
                    for (_, iter) in self.iterators.borrow().iter() {
                        let _ = iter.close(realm);
                    }
                    return Err(e);
                }
            }
        }

        drop(done_flags);
        drop(iterators);
        drop(padding);

        match self.mode {
            ZipMode::Shortest => {
                if any_done {
                    self.alive.set(false);
                    for (_, iter) in self.iterators.borrow().iter() {
                        let _ = iter.close(realm);
                    }
                    return create_iter_result_object(Value::Undefined, true, realm);
                }
            }
            ZipMode::Longest => {
                if all_done {
                    self.alive.set(false);
                    return create_iter_result_object(Value::Undefined, true, realm);
                }
            }
            ZipMode::Strict => {
                let done_flags = self.done.borrow();
                let first_done = done_flags.first().copied().unwrap_or(false);
                if any_done && !done_flags.iter().all(|&d| d == first_done) {
                    drop(done_flags);
                    self.alive.set(false);
                    for (_, iter) in self.iterators.borrow().iter() {
                        let _ = iter.close(realm);
                    }
                    return Err(Error::ty(
                        "Iterators have different lengths in strict mode",
                    ));
                }
                if all_done {
                    self.alive.set(false);
                    return create_iter_result_object(Value::Undefined, true, realm);
                }
            }
        }

        // Create object from entries
        let obj = Object::new(realm);
        for (key, value) in entries {
            obj.define_property(key.into(), value, realm)?;
        }
        create_iter_result_object(obj.into(), false, realm)
    }

    fn return_impl(&self, realm: &mut Realm) -> Res<ObjectHandle> {
        // Only close if we haven't already
        if self.alive.get() {
            self.alive.set(false);

            // Close all iterators in reverse order per IteratorCloseAll
            let mut last_error: Option<crate::Error> = None;
            for (_, iter) in self.iterators.borrow().iter().rev() {
                if let Err(e) = iter.close(realm) {
                    last_error = Some(e);
                }
            }
            if let Some(e) = last_error {
                return Err(e);
            }
        }

        create_iter_result_object(Value::Undefined, true, realm)
    }
}

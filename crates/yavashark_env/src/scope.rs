use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use yavashark_garbage::collectable::CellCollectable;
use yavashark_garbage::{Collectable, Gc, GcRef};
use yavashark_string::YSString;
use yavashark_value::CustomGcRefUntyped;

use crate::realm::Realm;
use crate::{Error, Object, ObjectHandle, Res, Value, Variable};

pub struct MutValue {
    pub name: String,
    pub scope: Rc<RefCell<ScopeInternal>>,
}

#[derive(Debug, Clone, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ScopeState {
    state: u8,
}

#[allow(unused)]
impl ScopeState {
    const NONE: u8 = 0b0;
    const FUNCTION: u8 = 0b10;
    const ITERATION: u8 = 0b100;
    const BREAKABLE: u8 = 0b1000;
    const RETURNABLE: u8 = 0b10000;
    const CONTINUABLE: u8 = 0b10_0000;
    const OPT_CHAIN: u8 = 0b100_0000;
    const STATE_NONE: Self = Self { state: Self::NONE };
    const STATE_FUNCTION: Self = Self {
        state: Self::FUNCTION,
    };
    const STATE_ITERATION: Self = Self {
        state: Self::ITERATION,
    };
    const STATE_BREAKABLE: Self = Self {
        state: Self::BREAKABLE,
    };
    const STATE_RETURNABLE: Self = Self {
        state: Self::RETURNABLE,
    };

    const STATE_CONTINUABLE: Self = Self {
        state: Self::CONTINUABLE,
    };

    const STATE_OPT_CHAIN: Self = Self {
        state: Self::OPT_CHAIN,
    };

    #[must_use]
    pub const fn new() -> Self {
        Self { state: 0 }
    }

    #[must_use]
    pub const fn copy(&self) -> Self {
        let mut state = self.state;

        state &= !Self::FUNCTION; // Remove the function state

        Self { state }
    }

    pub const fn set_function(&mut self) {
        self.state |= Self::FUNCTION;
        self.state |= Self::RETURNABLE;
    }

    pub const fn set_iteration(&mut self) {
        self.state |= Self::ITERATION;
        self.state |= Self::BREAKABLE;
    }

    pub const fn set_breakable(&mut self) {
        self.state |= Self::BREAKABLE;
    }

    pub const fn set_returnable(&mut self) {
        self.state |= Self::RETURNABLE;
    }

    pub const fn set_loop(&mut self) {
        self.state |= Self::CONTINUABLE;
        self.state |= Self::BREAKABLE;
    }

    pub const fn set_opt_chain(&mut self) {
        self.state |= Self::OPT_CHAIN;
    }

    #[must_use]
    pub const fn is_function(&self) -> bool {
        self.state & Self::FUNCTION != 0
    }

    #[must_use]
    pub const fn is_iteration(&self) -> bool {
        self.state & Self::ITERATION != 0
    }

    #[must_use]
    pub const fn is_breakable(&self) -> bool {
        self.state & Self::BREAKABLE != 0
    }

    #[must_use]
    pub const fn is_returnable(&self) -> bool {
        self.state & Self::RETURNABLE != 0
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.state == Self::NONE
    }

    #[must_use]
    pub const fn is_continuable(&self) -> bool {
        self.state & Self::CONTINUABLE != 0
    }
    #[must_use]
    pub const fn is_opt_chain(&self) -> bool {
        self.state & Self::OPT_CHAIN != 0
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    scope: Gc<RefCell<ScopeInternal>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub default: Option<Value>,
    pub exports: ObjectHandle,
    pub path: PathBuf,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            default: None,
            exports: Object::null(),
            path: PathBuf::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModuleScope {
    pub scope: Scope,
    pub module: Module,
}

#[derive(Debug)]
pub struct VariableReference {
    name: Value,
    object: ObjectHandle,
}

#[derive(Debug)]
pub enum VariableOrRef {
    Variable(Variable),
    Ref(VariableReference),
}

impl VariableReference {
    #[must_use]
    pub fn get(&self) -> Variable {
        self.object
            .resolve_property_no_get_set(&self.name)
            .ok()
            .flatten()
            .map_or(Value::Undefined.into(), |p| {
                Variable::with_attributes(p.value, p.attributes)
            })
    }

    pub fn update(&self, value: Value) -> Res {
        self.object.define_property(self.name.clone(), value)
    }
}

impl VariableOrRef {
    #[must_use]
    pub fn get(&self) -> Variable {
        match self {
            Self::Variable(v) => v.clone(),
            Self::Ref(r) => r.get(),
        }
    }

    pub fn update(&mut self, value: Value) -> Res {
        match self {
            Self::Variable(v) => v.value = value,
            Self::Ref(r) => return r.update(value),
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ObjectOrVariables {
    Object(ObjectHandle),
    Variables(FxHashMap<String, VariableOrRef>),
}

impl From<Variable> for VariableOrRef {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}

impl From<VariableReference> for VariableOrRef {
    fn from(variable: VariableReference) -> Self {
        Self::Ref(variable)
    }
}

impl ObjectOrVariables {
    fn insert(&mut self, name: String, variable: Variable) -> Res {
        match self {
            Self::Object(o) => o.define_variable(name.into(), variable)?,
            Self::Variables(v) => {
                v.insert(name, variable.into());
            }
        }

        Ok(())
    }

    fn insert_opt(&mut self, name: String, variable: Variable) -> Res {
        match self {
            Self::Object(o) => o.define_variable(name.into(), variable)?,
            Self::Variables(v) => {
                let entry = v.entry(name);

                match entry {
                    Entry::Occupied(mut occ) => {
                        if !variable.value.is_undefined() {
                            occ.insert(variable.into());
                        }
                    }
                    Entry::Vacant(vac) => {
                        vac.insert(variable.into());
                    }
                }
            }
        }

        Ok(())
    }

    fn get(&self, name: &str) -> Option<Variable> {
        match self {
            Self::Object(o) => o
                .resolve_property_no_get_set(&YSString::from_ref(name).into())
                .ok()
                .flatten()
                .map(|x| Variable::with_attributes(x.value, x.attributes)),
            Self::Variables(v) => v.get(name).map(VariableOrRef::get),
        }
    }

    fn contains_key(&self, name: &str) -> bool {
        match self {
            Self::Object(o) => o
                .contains_key(&YSString::from_ref(name).into())
                .unwrap_or_default(),
            Self::Variables(v) => v.contains_key(name),
        }
    }

    fn keys(&self) -> Vec<YSString> {
        match self {
            Self::Object(o) => o
                .keys()
                .map(|x| {
                    x.iter()
                        .filter_map(|v| {
                            if let Value::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Self::Variables(v) => v.keys().map(|k| YSString::from_ref(k)).collect(),
        }
    }
}

#[derive(Debug)]
pub struct ScopeInternal {
    parent: Option<Gc<RefCell<ScopeInternal>>>,
    variables: ObjectOrVariables,
    hoisted: HashSet<String>,
    pub available_labels: Vec<String>,
    pub last_label_is_current: bool,
    pub state: ScopeState,
    pub this: Value,
    pub new_target: Value,
    pub file: Option<PathBuf>,
}

unsafe impl CellCollectable<RefCell<Self>> for ScopeInternal {
    fn get_refs(&self) -> Vec<GcRef<RefCell<Self>>> {
        let mut refs = match &self.variables {
            ObjectOrVariables::Object(o) => vec![o.get_untyped_ref()],
            ObjectOrVariables::Variables(v) => {
                let mut refs = Vec::with_capacity(v.len());

                for v in v.values() {
                    if let Value::Object(o) = &v.get().value {
                        refs.push(o.get_untyped_ref());
                    }
                }

                if let Some(this) = self.this.gc_untyped_ref() {
                    refs.push(this);
                }

                refs
            }
        };

        if let Some(p) = &self.parent {
            refs.push(p.get_ref());
        }

        refs
    }
}

impl ScopeInternal {
    #[must_use]
    pub fn new(realm: &Realm, path: PathBuf) -> Self {
        let global = realm.global.clone();

        Self {
            parent: None,
            variables: ObjectOrVariables::Object(global),
            hoisted: HashSet::new(),
            available_labels: Vec::new(),
            last_label_is_current: false,
            state: ScopeState::new(),
            this: Value::Undefined,
            new_target: Value::Undefined,
            file: Some(path),
        }
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn global(realm: &Realm, path: PathBuf) -> Self {
        Self {
            parent: None,
            variables: ObjectOrVariables::Object(realm.global.clone()),
            hoisted: HashSet::new(),
            available_labels: Vec::new(),
            last_label_is_current: false,
            state: ScopeState::STATE_NONE,
            this: realm.global.clone().into(),
            new_target: Value::Undefined,
            file: Some(path),
        }
    }

    pub fn with_parent(parent: Gc<RefCell<Self>>) -> Res<Self> {
        let mut variables = FxHashMap::with_capacity_and_hasher(8, Default::default());

        variables.insert(
            "undefined".to_string(),
            Variable::new_read_only(Value::Undefined).into(),
        );
        variables.insert(
            "NaN".to_string(),
            Variable::new_read_only(Value::Number(f64::NAN)).into(),
        );
        variables.insert(
            "Infinity".to_string(),
            Variable::new_read_only(Value::Number(f64::INFINITY)).into(),
        );
        variables.insert(
            "null".to_string(),
            Variable::new_read_only(Value::Null).into(),
        );
        variables.insert(
            "true".to_string(),
            Variable::new_read_only(Value::Boolean(true)).into(),
        );
        variables.insert(
            "false".to_string(),
            Variable::new_read_only(Value::Boolean(false)).into(),
        );

        let state = parent.borrow()?.state.copy();

        let par_scope = parent.borrow()?;
        let available_labels = par_scope.available_labels.clone();

        let this = par_scope.this.copy();
        let new_target = par_scope.new_target.copy();
        drop(par_scope);

        Ok(Self {
            parent: Some(parent),
            variables: ObjectOrVariables::Variables(variables),
            hoisted: HashSet::new(),
            available_labels,
            last_label_is_current: false,
            state,
            this,
            new_target,
            file: None,
        })
    }

    pub fn with_parent_this(parent: Gc<RefCell<Self>>, this: Value) -> Res<Self> {
        let mut new = Self::with_parent(parent)?;

        new.this = this;

        Ok(new)
    }

    pub fn declare_var(&mut self, name: String, value: Value) -> Res {
        self.variables.insert_opt(name, Variable::new(value))
    }

    pub fn declare_read_only_var(&mut self, name: String, value: Value) -> Res {
        if self.variables.contains_key(&name) {
            return Err(Error::new("Variable already declared"));
        }

        self.variables
            .insert(name, Variable::new_read_only(value))?;

        Ok(())
    }

    pub fn declare_global_var(&mut self, name: String, value: Value) -> Res {
        #[allow(clippy::if_same_then_else)]
        if let ObjectOrVariables::Object(obj) = &mut self.variables {
            obj.define_property(name.into(), value)?;
        } else if self.state.is_function() {
            self.variables.insert_opt(name, Variable::new(value))?;
        } else {
            match &self.parent {
                Some(p) => {
                    p.borrow_mut()?.declare_global_var(name, value.copy())?;
                }
                None => {
                    self.variables.insert_opt(name, Variable::new(value))?;
                }
            }
        }

        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Res<Option<Value>> {
        if let Some(v) = self.variables.get(name) {
            return Ok(Some(v.copy()));
        }

        if self.hoisted.contains(name) {
            return Err(Error::reference_error(format!(
                "Cannot access {name} before initialization"
            )));
        }

        match &self.parent {
            Some(parent) => parent.borrow()?.resolve(name),
            None => Ok(None),
        }
    }

    pub fn has_value(&self, name: &str) -> Res<bool> {
        if self.variables.contains_key(name) {
            Ok(true)
        } else {
            match &self.parent {
                Some(parent) => parent.borrow()?.has_value(name),
                None => Ok(false),
            }
        }
    }

    #[must_use]
    pub fn has_label(&self, label: &str) -> bool {
        self.available_labels.contains(&label.to_string())
    }

    pub fn declare_label(&mut self, label: String) {
        self.available_labels.push(label);
        self.last_label_is_current = true;
    }

    pub fn last_label(&mut self) -> Option<&String> {
        if !self.last_label_is_current {
            return None;
        }
        self.available_labels.last()
    }

    pub const fn set_no_label(&mut self) {
        self.last_label_is_current = false;
    }

    pub const fn state_set_function(&mut self) {
        self.state.set_function();
    }

    pub const fn state_set_iteration(&mut self) {
        self.state.set_iteration();
    }

    pub const fn state_set_breakable(&mut self) {
        self.state.set_breakable();
    }

    pub const fn state_set_returnable(&mut self) {
        self.state.set_returnable();
    }

    pub const fn state_set_loop(&mut self) {
        self.state.set_loop();
    }

    pub const fn state_set_opt_chain(&mut self) {
        self.state.set_opt_chain();
    }

    #[must_use]
    pub const fn state_is_function(&self) -> bool {
        self.state.is_function()
    }

    #[must_use]
    pub const fn state_is_iteration(&self) -> bool {
        self.state.is_iteration()
    }

    #[must_use]
    pub const fn state_is_breakable(&self) -> bool {
        self.state.is_breakable()
    }

    #[must_use]
    pub const fn state_is_returnable(&self) -> bool {
        self.state.is_returnable()
    }

    #[must_use]
    pub const fn state_is_none(&self) -> bool {
        self.state.is_none()
    }

    #[must_use]
    pub const fn state_is_continuable(&self) -> bool {
        self.state.is_continuable()
    }

    #[must_use]
    pub const fn state_is_opt_chain(&self) -> bool {
        self.state.is_opt_chain()
    }

    pub fn update(&mut self, name: &str, value: Value) -> Res<bool> {
        match &mut self.variables {
            ObjectOrVariables::Object(obj) => {
                let name = YSString::from_ref(name).into();
                if let Ok(Some(prop)) = obj.resolve_property_no_get_set(&name) {
                    if !prop.attributes.is_writable() {
                        return Ok(false);
                    }

                    obj.define_variable(name, Variable::with_attributes(value, prop.attributes))?;
                    return Ok(true);
                }
            }
            ObjectOrVariables::Variables(ref mut v) => {
                if let Some(var) = v.get_mut(name) {
                    if !var.get().properties.is_writable() {
                        return Ok(false);
                    }

                    var.update(value)?;
                    return Ok(true);
                }
            }
        }

        if let Some(p) = &self.parent {
            return p.borrow_mut()?.update(name, value);
        }

        Ok(false)
    }

    pub fn update_or_define(&mut self, name: String, value: Value) -> Res {
        match &mut self.variables {
            ObjectOrVariables::Object(obj) => {
                let name = name.clone().into();
                if let Ok(Some(prop)) = obj.resolve_property_no_get_set(&name) {
                    if !prop.attributes.is_writable() {
                        return Err(Error::ty("Assignment to constant variable"));
                    }

                    obj.define_variable(name, Variable::with_attributes(value, prop.attributes))?;
                    return Ok(());
                }
            }
            ObjectOrVariables::Variables(v) => {
                if let Some(var) = v.get_mut(&name) {
                    if !var.get().properties.is_writable() {
                        return Err(Error::ty("Assignment to constant variable"));
                    }

                    var.update(value)?;
                    return Ok(());
                }
            }
        }

        if let Some(p) = &self.parent {
            if p.borrow_mut()?.update(&name, value.copy())? {
                return Ok(());
            }
        }

        self.declare_var(name, value)?;

        Ok(())
    }

    pub fn set_file(&mut self, file: PathBuf) {
        self.file = Some(file);
    }

    pub fn get_current_file(&self) -> Res<PathBuf> {
        if let Some(f) = self.file.clone() {
            return Ok(f);
        }

        match &self.parent {
            Some(p) => p.borrow()?.get_current_file(),
            None => Ok(PathBuf::new()),
        }
    }

    pub fn copy_path(&mut self) {
        self.file = self.get_current_file().ok();
    }

    pub fn get_variables(&self) -> Res<FxHashMap<String, Variable>> {
        let mut variables: FxHashMap<String, Variable> = match &self.parent {
            Some(p) => p.borrow()?.get_variables()?,
            None => FxHashMap::default(),
        };

        match &self.variables {
            ObjectOrVariables::Object(o) => {
                for (k, v) in &o.properties()? {
                    if let Value::String(s) = k {
                        variables.insert(s.to_string(), v.clone().into());
                    }
                }
            }
            ObjectOrVariables::Variables(v) => {
                for (name, variable) in v {
                    variables.insert(name.clone(), variable.get());
                }
            }
        }

        Ok(variables)
    }

    pub fn get_variable_names(&self) -> Res<HashSet<String>> {
        let mut variables = match &self.parent {
            Some(p) => p.borrow()?.get_variable_names()?,
            None => HashSet::new(),
        };

        variables.extend(self.variables.keys().iter().map(ToString::to_string));

        Ok(variables)
    }

    pub fn hoist(&mut self, name: String) {
        self.hoisted.insert(name);
    }

    pub fn is_hoisted(&self, name: &str) -> bool {
        self.hoisted.contains(name)
    }
}

impl Scope {
    #[must_use]
    pub fn new(realm: &Realm, path: PathBuf) -> Self {
        Self {
            scope: Gc::new(RefCell::new(ScopeInternal::new(realm, path))),
        }
    }

    #[must_use]
    pub fn global(realm: &Realm, path: PathBuf) -> Self {
        Self {
            scope: Gc::new(RefCell::new(ScopeInternal::global(realm, path))),
        }
    }

    pub fn with_parent(parent: &Self) -> Res<Self> {
        Ok(Self {
            scope: Gc::new(RefCell::new(ScopeInternal::with_parent(Gc::clone(
                &parent.scope,
            ))?)),
        })
    }

    pub fn object_with_parent(parent: &Self, object: ObjectHandle) -> Res<Self> {
        let borrow = parent.scope.borrow()?;

        let this = borrow.this.copy();
        let new_target = borrow.new_target.copy();

        Ok(Self {
            scope: Gc::new(RefCell::new(ScopeInternal {
                parent: Some(Gc::clone(&parent.scope)),
                variables: ObjectOrVariables::Object(object),
                hoisted: HashSet::new(),
                available_labels: Vec::new(),
                last_label_is_current: false,
                state: ScopeState::new(),
                this,
                new_target,
                file: None,
            })),
        })
    }

    pub fn with_parent_this(parent: &Self, this: Value) -> Res<Self> {
        Ok(Self {
            scope: Gc::new(RefCell::new(ScopeInternal::with_parent_this(
                Gc::clone(&parent.scope),
                this,
            )?)),
        })
    }

    pub fn declare_var(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut()?.declare_var(name, value)
    }

    pub fn declare_read_only_var(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut()?.declare_read_only_var(name, value)
    }

    pub fn declare_global_var(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut()?.declare_global_var(name, value)?;
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Res<Option<Value>> {
        self.scope.borrow()?.resolve(name)
    }

    pub fn has_label(&self, label: &str) -> Res<bool> {
        let Ok(scope) = self.scope.borrow() else {
            return Ok(false);
        };

        if scope.has_label(label) {
            Ok(true)
        } else {
            if let Some(p) = &scope.parent {
                return Ok(p.borrow()?.has_label(label));
            }
            Ok(false)
        }
    }

    pub fn declare_label(&mut self, label: String) -> Res {
        self.scope.borrow_mut()?.declare_label(label);
        Ok(())
    }

    pub fn last_label(&self) -> Res<Option<String>> {
        Ok(self.scope.borrow_mut()?.last_label().cloned())
    }

    pub fn set_no_label(&self) -> Res {
        self.scope.borrow_mut()?.set_no_label();

        Ok(())
    }

    #[must_use]
    pub fn state(&self) -> ScopeState {
        self.scope
            .borrow()
            .map(|x| x.state.clone())
            .unwrap_or_default()
    }

    pub fn state_set_function(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_function();
        Ok(())
    }

    pub fn state_set_iteration(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_iteration();
        Ok(())
    }

    pub fn state_set_breakable(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_breakable();
        Ok(())
    }

    pub fn state_set_returnable(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_returnable();
        Ok(())
    }

    pub fn set_target(&mut self, target: Value) -> Res {
        self.scope.borrow_mut()?.new_target = target;
        Ok(())
    }

    pub fn get_target(&self) -> Res<Value> {
        Ok(self.scope.borrow()?.new_target.copy())
    }

    pub fn state_set_loop(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_loop();
        Ok(())
    }

    pub fn state_set_opt_chain(&mut self) -> Res {
        self.scope.borrow_mut()?.state_set_opt_chain();
        Ok(())
    }

    pub fn state_is_function(&self) -> Res<bool> {
        Ok(self.scope.borrow()?.state_is_function())
    }

    pub fn state_is_iteration(&self) -> Res<bool> {
        Ok(self.scope.borrow()?.state_is_iteration())
    }

    pub fn state_is_breakable(&self) -> Res<bool> {
        Ok(self.scope.borrow()?.state_is_breakable())
    }

    pub fn state_is_returnable(&self) -> Res<bool> {
        Ok(self.scope.borrow()?.state_is_returnable())
    }

    pub fn state_is_none(&self) -> Res<bool> {
        Ok(self.scope.borrow()?.state_is_none())
    }

    pub fn state_is_continuable(&self) -> Res<bool> {
        Ok(self.scope.borrow()?.state_is_continuable())
    }

    pub fn state_is_opt_chain(&self) -> Res<bool> {
        Ok(self.scope.borrow()?.state_is_opt_chain())
    }

    pub fn has_value(&self, name: &str) -> Res<bool> {
        self.scope.borrow()?.has_value(name)
    }

    pub fn update(&mut self, name: &str, value: Value) -> Res<bool> {
        self.scope.borrow_mut()?.update(name, value)
    }

    pub fn update_or_define(&mut self, name: String, value: Value) -> Res {
        self.scope.borrow_mut()?.update_or_define(name, value)
    }

    pub fn child(&self) -> Res<Self> {
        Self::with_parent(self)
    }
    pub fn child_object(&self, object: ObjectHandle) -> Res<Self> {
        Self::object_with_parent(self, object)
    }

    pub fn this(&self) -> Res<Value> {
        Ok(self.scope.borrow()?.this.copy())
    }

    pub fn parent(&self) -> Res<Option<Gc<RefCell<ScopeInternal>>>> {
        Ok(self.scope.borrow()?.parent.clone())
    }

    pub fn set_path(&mut self, path: PathBuf) -> Res {
        self.scope.borrow_mut()?.set_file(path);

        Ok(())
    }

    pub fn get_current_path(&self) -> Res<PathBuf> {
        self.scope.borrow()?.get_current_file()
    }

    pub fn copy_path(&self) -> Res {
        self.scope.borrow_mut()?.copy_path();

        Ok(())
    }

    pub fn get_variables(&self) -> Res<FxHashMap<String, Variable>> {
        self.scope.borrow()?.get_variables()
    }

    pub fn get_variable_names(&self) -> Res<HashSet<String>> {
        self.scope.borrow()?.get_variable_names()
    }

    #[must_use]
    pub fn into_module(self) -> ModuleScope {
        ModuleScope {
            scope: self,
            module: Module::default(),
        }
    }

    pub fn hoist(&self, name: String) -> Res {
        self.scope.borrow_mut()?.hoist(name);
        Ok(())
    }

    pub fn is_hoisted(&self, name: &str) -> Res<bool> {
        Ok(self.scope.borrow()?.is_hoisted(name))
    }
}

impl CustomGcRefUntyped for Scope {
    fn gc_untyped_ref<U: Collectable>(&self) -> Option<GcRef<U>> {
        Some(self.scope.get_untyped_ref())
    }
}

impl From<ScopeInternal> for Scope {
    fn from(scope: ScopeInternal) -> Self {
        Self {
            scope: Gc::new(RefCell::new(scope)),
        }
    }
}

impl From<Gc<RefCell<ScopeInternal>>> for Scope {
    fn from(scope: Gc<RefCell<ScopeInternal>>) -> Self {
        Self { scope }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_state_new_is_none() {
        let state = ScopeState::new();
        assert!(state.is_none());
    }

    #[test]
    fn scope_state_set_function_is_function() {
        let mut state = ScopeState::new();
        state.set_function();
        assert!(state.is_function());
    }

    #[test]
    fn scope_state_set_iteration_is_iteration() {
        let mut state = ScopeState::new();
        state.set_iteration();
        assert!(state.is_iteration());
    }

    #[test]
    fn scope_state_set_breakable_is_breakable() {
        let mut state = ScopeState::new();
        state.set_breakable();
        assert!(state.is_breakable());
    }

    #[test]
    fn scope_state_set_returnable_is_returnable() {
        let mut state = ScopeState::new();
        state.set_returnable();
        assert!(state.is_returnable());
    }

    #[test]
    fn scope_state_set_loop_is_continuable() {
        let mut state = ScopeState::new();
        state.set_loop();
        assert!(state.is_continuable());
    }

    #[test]
    fn scope_internal_declare_var_and_resolve() {
        let realm = Realm::new().unwrap();
        let mut scope = ScopeInternal::new(&realm, PathBuf::from("test.js"));
        scope
            .declare_var("test".to_string(), Value::Number(42.0))
            .unwrap();
        let value = scope.resolve("test").unwrap().unwrap();
        assert_eq!(value, Value::Number(42.0));
    }

    #[test]
    fn scope_internal_declare_read_only_var_and_update_fails() {
        let realm = Realm::new().unwrap();
        let mut scope = ScopeInternal::new(&realm, PathBuf::from("test.js"));
        scope
            .declare_read_only_var("test".to_string(), Value::Number(42.0))
            .unwrap();
        let result = scope.update("test", Value::Number(43.0)).unwrap();
        assert!(!result);
    }

    #[test]
    fn scope_internal_declare_global_var_and_resolve() {
        let realm = Realm::new().unwrap();
        let mut scope = ScopeInternal::new(&realm, PathBuf::from("test.js"));
        scope
            .declare_global_var("test".to_string(), Value::Number(42.0))
            .unwrap();
        let value = scope.resolve("test").unwrap().unwrap();
        assert_eq!(value, Value::Number(42.0));
    }

    #[test]
    fn scope_internal_update_or_define_and_resolve() {
        let realm = Realm::new().unwrap();
        let mut scope = ScopeInternal::new(&realm, PathBuf::from("test.js"));
        scope
            .update_or_define("test".to_string(), Value::Number(42.0))
            .unwrap();
        let value = scope.resolve("test").unwrap().unwrap();
        assert_eq!(value, Value::Number(42.0));
    }
}

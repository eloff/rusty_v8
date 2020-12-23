use crate::data::Data;
use crate::data::FunctionTemplate;
use crate::data::Name;
use crate::data::ObjectTemplate;
use crate::data::Template;
use crate::isolate::Isolate;
use crate::support::int;
use crate::support::MapFnTo;
use crate::CFunction;
use crate::ConstructorBehavior;
use crate::Context;
use crate::Function;
use crate::FunctionBuilder;
use crate::FunctionCallback;
use crate::HandleScope;
use crate::Local;
use crate::Object;
use crate::PropertyAttribute;
use crate::SideEffectType;
use crate::Signature;
use crate::String;
use crate::Value;
use crate::NONE;
use std::convert::TryFrom;
use std::ptr::null;

extern "C" {
  fn v8__Template__Set(
    this: *const Template,
    key: *const Name,
    value: *const Data,
    attr: PropertyAttribute,
  );

  fn v8__FunctionTemplate__New(
    isolate: *mut Isolate,
    callback: FunctionCallback,
    data_or_null: *const Value,
    signature_or_null: *const Signature,
    length: i32,
    constructor_behavior: ConstructorBehavior,
    side_effect_type: SideEffectType,
    c_function_or_null: *const CFunction,
  ) -> *const FunctionTemplate;
  fn v8__FunctionTemplate__GetFunction(
    this: *const FunctionTemplate,
    context: *const Context,
  ) -> *const Function;
  fn v8__FunctionTemplate__SetClassName(
    this: *const FunctionTemplate,
    name: *const String,
  );

  fn v8__ObjectTemplate__New(
    isolate: *mut Isolate,
    templ: *const FunctionTemplate,
  ) -> *const ObjectTemplate;
  fn v8__ObjectTemplate__NewInstance(
    this: *const ObjectTemplate,
    context: *const Context,
  ) -> *const Object;
  fn v8__ObjectTemplate__InternalFieldCount(this: *const ObjectTemplate)
    -> int;
  fn v8__ObjectTemplate__SetInternalFieldCount(
    this: *const ObjectTemplate,
    value: int,
  );
}

impl Template {
  /// Adds a property to each instance created by this template.
  pub fn set(&self, key: Local<Name>, value: Local<Data>) {
    self.set_with_attr(key, value, NONE)
  }

  /// Adds a property to each instance created by this template with
  /// the specified property attributes.
  pub fn set_with_attr(
    &self,
    key: Local<Name>,
    value: Local<Data>,
    attr: PropertyAttribute,
  ) {
    unsafe { v8__Template__Set(self, &*key, &*value, attr) }
  }
}

impl<'s> FunctionBuilder<'s, FunctionTemplate> {
  /// Set the function call signature. The default is no signature.
  pub fn signature(mut self, signature: Local<'s, Signature>) -> Self {
    self.signature = Some(signature);
    self
  }

  /// Creates the function template.
  pub fn build(
    self,
    scope: &mut HandleScope<'s, ()>,
  ) -> Local<'s, FunctionTemplate> {
    unsafe {
      scope.cast_local(|sd| {
        v8__FunctionTemplate__New(
          sd.get_isolate_ptr(),
          self.callback,
          self.data.map_or_else(null, |p| &*p),
          self.signature.map_or_else(null, |p| &*p),
          self.length,
          self.constructor_behavior,
          self.side_effect_type,
          null(),
        )
      })
    }
    .unwrap()
  }
}

impl FunctionTemplate {
  /// Create a FunctionBuilder to configure a FunctionTemplate.
  /// This is the same as FunctionBuilder::<FunctionTemplate>::new().
  pub fn builder<'s>(
    callback: impl MapFnTo<FunctionCallback>,
  ) -> FunctionBuilder<'s, Self> {
    FunctionBuilder::new(callback)
  }

  /// Creates a function template.
  pub fn new<'s>(
    scope: &mut HandleScope<'s, ()>,
    callback: impl MapFnTo<FunctionCallback>,
  ) -> Local<'s, FunctionTemplate> {
    Self::builder(callback).build(scope)
  }

  /// Returns the unique function instance in the current execution context.
  pub fn get_function<'s>(
    &self,
    scope: &mut HandleScope<'s>,
  ) -> Option<Local<'s, Function>> {
    unsafe {
      scope.cast_local(|sd| {
        v8__FunctionTemplate__GetFunction(self, sd.get_current_context())
      })
    }
  }

  /// Set the class name of the FunctionTemplate. This is used for
  /// printing objects created with the function created from the
  /// FunctionTemplate as its constructor.
  pub fn set_class_name(&self, name: Local<String>) {
    unsafe { v8__FunctionTemplate__SetClassName(self, &*name) };
  }
}

impl ObjectTemplate {
  /// Creates an object template.
  pub fn new<'s>(scope: &mut HandleScope<'s, ()>) -> Local<'s, ObjectTemplate> {
    unsafe {
      scope.cast_local(|sd| {
        v8__ObjectTemplate__New(sd.get_isolate_ptr(), std::ptr::null())
      })
    }
    .unwrap()
  }

  /// Creates an object template from a function template.
  pub fn new_from_template<'s>(
    scope: &mut HandleScope<'s, ()>,
    templ: Local<FunctionTemplate>,
  ) -> Local<'s, ObjectTemplate> {
    unsafe {
      scope
        .cast_local(|sd| v8__ObjectTemplate__New(sd.get_isolate_ptr(), &*templ))
    }
    .unwrap()
  }

  /// Creates a new instance of this object template.
  pub fn new_instance<'s>(
    &self,
    scope: &mut HandleScope<'s>,
  ) -> Option<Local<'s, Object>> {
    unsafe {
      scope.cast_local(|sd| {
        v8__ObjectTemplate__NewInstance(self, sd.get_current_context())
      })
    }
  }

  /// Gets the number of internal fields for objects generated from
  /// this template.
  pub fn internal_field_count(&self) -> usize {
    let count = unsafe { v8__ObjectTemplate__InternalFieldCount(self) };
    usize::try_from(count).expect("bad internal field count") // Can't happen.
  }

  /// Sets the number of internal fields for objects generated from
  /// this template.
  pub fn set_internal_field_count(&self, value: usize) -> bool {
    // The C++ API takes an i32 but trying to set a value < 0
    // results in unpredictable behavior, hence we disallow it.
    match int::try_from(value) {
      Err(_) => false,
      Ok(value) => {
        unsafe { v8__ObjectTemplate__SetInternalFieldCount(self, value) };
        true
      }
    }
  }
}

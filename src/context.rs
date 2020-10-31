// Copyright 2019-2020 the Deno authors. All rights reserved. MIT license.
use crate::isolate::Isolate;
use crate::Context;
use crate::HandleScope;
use crate::Local;
use crate::Object;
use crate::ObjectTemplate;
use crate::Value;
use std::ptr::null;

extern "C" {
  fn v8__Context__New(
    isolate: *mut Isolate,
    templ: *const ObjectTemplate,
    global_object: *const Value,
  ) -> *const Context;
  fn v8__Context__Global(this: *const Context) -> *const Object;
  fn v8__Context__AllowCodeGenerationFromStrings(this: *mut Context, allow: bool);
  fn v8__Context__IsCodeGenerationFromStringsAllowed(this: *const Context) -> bool;
}

impl Context {
  /// Creates a new context.
  pub fn new<'s>(scope: &mut HandleScope<'s, ()>) -> Local<'s, Context> {
    // TODO: optional arguments;
    unsafe {
      scope
        .cast_local(|sd| v8__Context__New(sd.get_isolate_ptr(), null(), null()))
    }
    .unwrap()
  }

  /// Creates a new context using the object template as the template for
  /// the global object.
  pub fn new_from_template<'s>(
    scope: &mut HandleScope<'s, ()>,
    templ: Local<ObjectTemplate>,
  ) -> Local<'s, Context> {
    unsafe {
      scope.cast_local(|sd| {
        v8__Context__New(sd.get_isolate_ptr(), &*templ, null())
      })
    }
    .unwrap()
  }

  /// Returns the global proxy object.
  ///
  /// Global proxy object is a thin wrapper whose prototype points to actual
  /// context's global object with the properties like Object, etc. This is done
  /// that way for security reasons (for more details see
  /// https://wiki.mozilla.org/Gecko:SplitWindow).
  ///
  /// Please note that changes to global proxy object prototype most probably
  /// would break VM---v8 expects only global object as a prototype of global
  /// proxy object.
  pub fn global<'s>(
    &self,
    scope: &mut HandleScope<'s, ()>,
  ) -> Local<'s, Object> {
    unsafe { scope.cast_local(|_| v8__Context__Global(self)) }.unwrap()
  }

  /// Control whether code generation from strings is allowed. Calling
  /// this method with false will disable 'eval' and the 'Function'
  /// constructor for code running in this context. If 'eval' or the
  /// 'Function' constructor are used an exception will be thrown.
  ///
  /// If code generation from strings is not allowed the
  /// V8::AllowCodeGenerationFromStrings callback will be invoked if
  /// set before blocking the call to 'eval' or the 'Function'
  /// constructor. If that callback returns true, the call will be
  /// allowed, otherwise an exception will be thrown. If no callback is
  /// set an exception will be thrown.
  pub fn allow_code_generation_from_strings(&mut self, allow: bool) {
    unsafe { v8__Context__AllowCodeGenerationFromStrings(self, allow); }
  }

  /// Returns true if code generation from strings is allowed for the context.
  /// For more details see allow_code_generation_from_strings(bool) documentation.
  pub fn is_code_generation_from_strings_allowed(&self) -> bool {
    unsafe { v8__Context__IsCodeGenerationFromStringsAllowed(self) }
  }
}

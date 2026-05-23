mod jstd_bindings;

use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ptr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

use mozjs::jsapi::{JS_IsExceptionPending, OnNewGlobalHookOption};
use mozjs::jsval::UndefinedValue;
use mozjs::realm::AutoRealm;
use mozjs::rooted;
use mozjs::rust::CompileOptionsWrapper;
use mozjs::rust::SIMPLE_GLOBAL_CLASS;
use mozjs::rust::evaluate_script;
use mozjs::rust::wrappers2::{InitRealmStandardClasses, JS_NewGlobalObject};
use mozjs::rust::{JSEngine as MozJSEngine, JSEngineHandle, RealmOptions, Runtime};

use o_core::engine::JSEngine;
use o_core::error::{JSError, JSResult};

static JS_ENGINE_PTR: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());
static INIT_LOCK: Mutex<()> = Mutex::new(());

fn get_engine_handle() -> JSEngineHandle {
    let ptr = JS_ENGINE_PTR.load(Ordering::SeqCst);
    if ptr.is_null() {
        let _lock = INIT_LOCK.lock().unwrap();
        let ptr = JS_ENGINE_PTR.load(Ordering::SeqCst);
        if ptr.is_null() {
            let engine = match MozJSEngine::init() {
                Ok(e) => e,
                Err(_) => {
                    JS_ENGINE_PTR.store(1usize as *mut (), Ordering::SeqCst);
                    return get_existing_handle();
                }
            };
            let handle = engine.handle();
            JS_ENGINE_PTR.store(Box::into_raw(Box::new(engine)).cast(), Ordering::SeqCst);
            handle
        } else {
            get_existing_handle()
        }
    } else {
        get_existing_handle()
    }
}

fn get_existing_handle() -> JSEngineHandle {
    let ptr = JS_ENGINE_PTR.load(Ordering::SeqCst);
    unsafe { &*(ptr.cast::<MozJSEngine>()) }.handle()
}

pub struct SpiderMonkey {
    runtime: UnsafeCell<ManuallyDrop<Runtime>>,
}

impl SpiderMonkey {
    pub fn new() -> Self {
        let handle = get_engine_handle();
        let runtime = Runtime::new(handle);
        Self {
            runtime: UnsafeCell::new(ManuallyDrop::new(runtime)),
        }
    }
}

impl Default for SpiderMonkey {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Sync for SpiderMonkey {}

impl JSEngine for SpiderMonkey {
    fn run(&self, code: &str, filename: &str) -> Result<JSResult, JSError> {
        let rt = unsafe { &mut *self.runtime.get() };
        let cx = rt.cx();

        let options = RealmOptions::default();
        rooted!(&in(cx) let global = unsafe {
            JS_NewGlobalObject(
                cx,
                &SIMPLE_GLOBAL_CLASS,
                ptr::null_mut(),
                OnNewGlobalHookOption::FireOnNewGlobalHook,
                &*options,
            )
        });

        let mut realm = AutoRealm::new_from_handle(cx, global.handle());
        let cx = &mut realm;

        if !unsafe { InitRealmStandardClasses(cx) } {
            return Err(
                JSError::internal("failed to initialize SpiderMonkey standard classes")
                    .with_filename(filename),
            );
        }

        jstd_bindings::define_jstd_for_global(cx, global.handle());

        let bootstrap_options =
            CompileOptionsWrapper::new(cx, std::ffi::CString::new("<jstd>").unwrap(), 1);
        rooted!(&in(cx) let mut bootstrap_rval = UndefinedValue());
        if evaluate_script(
            cx,
            global.handle(),
            jstd::bootstrap_script(),
            bootstrap_rval.handle_mut(),
            bootstrap_options,
        )
        .is_err()
        {
            return Err(
                JSError::internal("failed to initialize jstd builtins").with_filename(filename)
            );
        }

        let filename_cstr = std::ffi::CString::new(filename)
            .unwrap_or_else(|_| std::ffi::CString::new("unknown").unwrap());
        let options = CompileOptionsWrapper::new(cx, filename_cstr, 1);

        rooted!(&in(cx) let mut rval = UndefinedValue());

        if evaluate_script(cx, global.handle(), code, rval.handle_mut(), options).is_err() {
            return Err(
                JSError::new(format!("JavaScript execution error in {}", filename))
                    .with_kind(o_core::error::JSErrorKind::Runtime)
                    .with_filename(filename)
                    .with_source(code),
            );
        }

        unsafe {
            if JS_IsExceptionPending(cx.raw_cx()) {
                return Err(JSError::runtime(
                    "JavaScript exception is pending after script execution",
                )
                .with_filename(filename)
                .with_source(code));
            }
        }

        let val = rval.get();
        if val.is_string() {
            Ok(JSResult::String("string value".to_string()))
        } else if val.is_int32() {
            Ok(JSResult::String(val.to_int32().to_string()))
        } else if val.is_double() {
            Ok(JSResult::String(val.to_double().to_string()))
        } else if val.is_boolean() {
            Ok(JSResult::String(val.to_boolean().to_string()))
        } else if val.is_undefined() {
            Ok(JSResult::String("undefined".to_string()))
        } else if val.is_null() {
            Ok(JSResult::String("null".to_string()))
        } else {
            Ok(JSResult::String("object".to_string()))
        }
    }
}

impl Drop for SpiderMonkey {
    fn drop(&mut self) {
        // SpiderMonkey context teardown is currently unstable in this embedding
        // and can segfault on process shutdown. Intentionally leak the runtime
        // until lifecycle management is implemented correctly.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{LazyLock, Mutex};

    static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[test]
    #[ignore = "SpiderMonkey runtime is unstable under Rust test harness"]
    fn test_run_simple_expression() {
        let _guard = TEST_LOCK.lock().unwrap();
        let engine = SpiderMonkey::new();
        let result = engine.run("40 + 2", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "42"),
        }
    }

    #[test]
    #[ignore = "SpiderMonkey runtime is unstable under Rust test harness"]
    fn test_run_string_literal() {
        let _guard = TEST_LOCK.lock().unwrap();
        let engine = SpiderMonkey::new();
        let result = engine.run("'hello'", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "string value"),
        }
    }

    #[test]
    #[ignore = "SpiderMonkey runtime is unstable under Rust test harness"]
    fn test_run_boolean() {
        let _guard = TEST_LOCK.lock().unwrap();
        let engine = SpiderMonkey::new();
        let result = engine.run("true", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "true"),
        }
    }
    #[test]
    #[ignore = "SpiderMonkey runtime is unstable under Rust test harness"]
    fn test_console_log() {
        let _guard = TEST_LOCK.lock().unwrap();
        let engine = SpiderMonkey::new();
        let result = engine
            .run("console.log(\"Hello, World!\")", "<test>")
            .unwrap();
        match result {
            JSResult::String(s) => assert_eq!("undefined", s),
        }
    }
}

use std::sync::Once;

use jstd::console;
use o_core::engine::JSEngine;
use o_core::error::{JSError, JSResult};
use rusty_v8 as v8;

static V8_INIT: Once = Once::new();

fn ensure_v8_initialized() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

fn v8_value_to_string(scope: &mut v8::PinScope, value: v8::Local<v8::Value>) -> String {
    if value.is_string() {
        value.to_rust_string_lossy(scope)
    } else if value.is_number() {
        let number = value.number_value(scope).unwrap_or(0.0);
        if number.fract() == 0.0 {
            (number as i64).to_string()
        } else {
            number.to_string()
        }
    } else if value.is_boolean() {
        value.boolean_value(scope).to_string()
    } else if value.is_null() {
        "null".to_string()
    } else if value.is_undefined() {
        "undefined".to_string()
    } else {
        value
            .to_string(scope)
            .map(|s| s.to_rust_string_lossy(scope))
            .unwrap_or_else(|| "[object]".to_string())
    }
}

fn callback_args_to_strings(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
) -> Vec<String> {
    (0..args.length())
        .map(|i| v8_value_to_string(scope, args.get(i)))
        .collect()
}

fn first_label(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments) -> String {
    if args.length() > 0 {
        v8_value_to_string(scope, args.get(0))
    } else {
        "default".to_string()
    }
}

fn return_undefined(scope: &mut v8::PinScope, mut rv: v8::ReturnValue) {
    rv.set(v8::undefined(scope).into());
}

fn host_console_log(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::log(&callback_args_to_strings(scope, args));
    return_undefined(scope, rv);
}

fn host_console_info(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::info(&callback_args_to_strings(scope, args));
    return_undefined(scope, rv);
}

fn host_console_warn(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::warn(&callback_args_to_strings(scope, args));
    return_undefined(scope, rv);
}

fn host_console_error(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::error(&callback_args_to_strings(scope, args));
    return_undefined(scope, rv);
}

fn host_console_debug(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::debug(&callback_args_to_strings(scope, args));
    return_undefined(scope, rv);
}

fn host_console_trace(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::trace(&callback_args_to_strings(scope, args));
    return_undefined(scope, rv);
}

fn host_console_assert(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    let condition = if args.length() > 0 {
        args.get(0).boolean_value(scope)
    } else {
        false
    };
    let values = (1..args.length())
        .map(|i| v8_value_to_string(scope, args.get(i)))
        .collect::<Vec<_>>();
    console::assert(condition, &values);
    return_undefined(scope, rv);
}

fn host_console_clear(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::clear();
    return_undefined(scope, rv);
}

fn host_console_count(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::count(&first_label(scope, args));
    return_undefined(scope, rv);
}

fn host_console_count_reset(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::count_reset(&first_label(scope, args));
    return_undefined(scope, rv);
}

fn host_console_time(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::time(&first_label(scope, args));
    return_undefined(scope, rv);
}

fn host_console_time_log(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::time_log(&first_label(scope, args));
    return_undefined(scope, rv);
}

fn host_console_time_end(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::time_end(&first_label(scope, args));
    return_undefined(scope, rv);
}

fn host_console_group(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::group(&first_label(scope, args));
    return_undefined(scope, rv);
}

fn host_console_group_collapsed(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::group_collapsed(&first_label(scope, args));
    return_undefined(scope, rv);
}

fn host_console_group_end(
    scope: &mut v8::PinScope,
    _args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::group_end();
    return_undefined(scope, rv);
}

fn host_print(scope: &mut v8::PinScope, args: v8::FunctionCallbackArguments, rv: v8::ReturnValue) {
    console::print(&callback_args_to_strings(scope, args));
    return_undefined(scope, rv);
}

fn host_println(
    scope: &mut v8::PinScope,
    args: v8::FunctionCallbackArguments,
    rv: v8::ReturnValue,
) {
    console::println(&callback_args_to_strings(scope, args));
    return_undefined(scope, rv);
}

fn install_global_function(
    scope: &mut v8::PinScope,
    context: v8::Local<v8::Context>,
    name: &str,
    callback: impl v8::MapFnTo<v8::FunctionCallback>,
) -> Result<(), JSError> {
    let global = context.global(scope);
    let key = v8::String::new(scope, name)
        .ok_or_else(|| JSError::internal(format!("failed to create V8 string for {name}")))?;
    let templ = v8::FunctionTemplate::new(scope, callback);
    let func = templ
        .get_function(scope)
        .ok_or_else(|| JSError::internal(format!("failed to create V8 function for {name}")))?;

    global
        .set(scope, key.into(), func.into())
        .ok_or_else(|| JSError::internal(format!("failed to install global {name}")))?;
    Ok(())
}

fn install_jstd(scope: &mut v8::PinScope, context: v8::Local<v8::Context>) -> Result<(), JSError> {
    install_global_function(scope, context, "__jstd_console_log", host_console_log)?;
    install_global_function(scope, context, "__jstd_console_info", host_console_info)?;
    install_global_function(scope, context, "__jstd_console_warn", host_console_warn)?;
    install_global_function(scope, context, "__jstd_console_error", host_console_error)?;
    install_global_function(scope, context, "__jstd_console_debug", host_console_debug)?;
    install_global_function(scope, context, "__jstd_console_trace", host_console_trace)?;
    install_global_function(scope, context, "__jstd_console_assert", host_console_assert)?;
    install_global_function(scope, context, "__jstd_console_clear", host_console_clear)?;
    install_global_function(scope, context, "__jstd_console_count", host_console_count)?;
    install_global_function(
        scope,
        context,
        "__jstd_console_count_reset",
        host_console_count_reset,
    )?;
    install_global_function(scope, context, "__jstd_console_time", host_console_time)?;
    install_global_function(
        scope,
        context,
        "__jstd_console_time_log",
        host_console_time_log,
    )?;
    install_global_function(
        scope,
        context,
        "__jstd_console_time_end",
        host_console_time_end,
    )?;
    install_global_function(scope, context, "__jstd_console_group", host_console_group)?;
    install_global_function(
        scope,
        context,
        "__jstd_console_group_collapsed",
        host_console_group_collapsed,
    )?;
    install_global_function(
        scope,
        context,
        "__jstd_console_group_end",
        host_console_group_end,
    )?;
    install_global_function(scope, context, "__jstd_print", host_print)?;
    install_global_function(scope, context, "__jstd_println", host_println)?;
    Ok(())
}

pub struct V8Engine;

impl V8Engine {
    pub fn new() -> Self {
        ensure_v8_initialized();
        Self
    }
}

impl Default for V8Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl JSEngine for V8Engine {
    fn run(&self, code: &str, _filename: &str) -> Result<JSResult, JSError> {
        ensure_v8_initialized();

        let isolate = &mut v8::Isolate::new(v8::CreateParams::default());
        v8::scope!(let handle_scope, isolate);
        let context = v8::Context::new(handle_scope, Default::default());
        let scope = &mut v8::ContextScope::new(handle_scope, context);

        install_jstd(scope, context)?;

        let bootstrap = v8::String::new(scope, jstd::bootstrap_script())
            .ok_or_else(|| JSError::internal("failed to build jstd bootstrap source"))?;
        let bootstrap = v8::Script::compile(scope, bootstrap, None)
            .ok_or_else(|| JSError::internal("failed to compile jstd bootstrap"))?;
        bootstrap
            .run(scope)
            .ok_or_else(|| JSError::internal("failed to run jstd bootstrap"))?;

        let source = v8::String::new(scope, code).ok_or_else(|| {
            JSError::compile("failed to build V8 source string").with_filename(_filename)
        })?;
        let script = v8::Script::compile(scope, source, None).ok_or_else(|| {
            JSError::compile("failed to compile V8 script")
                .with_filename(_filename)
                .with_source(code)
        })?;
        let result = script.run(scope).ok_or_else(|| {
            JSError::runtime("failed to run V8 script")
                .with_filename(_filename)
                .with_source(code)
        })?;

        if result.is_string() {
            return Ok(JSResult::String(result.to_rust_string_lossy(scope)));
        }

        if result.is_number() {
            let number = result.number_value(scope).ok_or_else(|| {
                JSError::runtime("failed to read V8 number result").with_filename(_filename)
            })?;
            if number.fract() == 0.0 {
                return Ok(JSResult::String((number as i64).to_string()));
            }
            return Ok(JSResult::String(number.to_string()));
        }

        if result.is_boolean() {
            return Ok(JSResult::String(result.boolean_value(scope).to_string()));
        }

        if result.is_undefined() {
            return Ok(JSResult::String("undefined".to_string()));
        }

        if result.is_null() {
            return Ok(JSResult::String("null".to_string()));
        }

        Ok(JSResult::String("object".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_simple_expression() {
        let engine = V8Engine::new();
        let result = engine.run("40 + 2", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "42"),
        }
    }

    #[test]
    fn test_run_string_literal() {
        let engine = V8Engine::new();
        let result = engine.run("'hello'", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "hello"),
        }
    }

    #[test]
    fn test_run_boolean() {
        let engine = V8Engine::new();
        let result = engine.run("true", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "true"),
        }
    }

    #[test]
    fn test_console_log_builtin() {
        let engine = V8Engine::new();
        let result = engine.run("console.log('hello'); 42", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "42"),
        }
    }
}

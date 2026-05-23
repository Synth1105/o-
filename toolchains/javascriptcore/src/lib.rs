use jstd::console;
use o_core::engine::JSEngine;
use o_core::error::{JSError, JSResult};
use rust_jsc::{JSContext, JSFunction, JSObject, JSValue, PropertyDescriptor, callback};

type HostResult = rust_jsc::JSResult<JSValue>;

fn js_values_to_strings(arguments: &[JSValue]) -> Vec<String> {
    arguments
        .iter()
        .map(|value| {
            if value.is_string() {
                value
                    .as_string()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "[string]".to_string())
            } else if value.is_number() {
                value
                    .as_number()
                    .map(|n| {
                        if n.fract() == 0.0 {
                            (n as i64).to_string()
                        } else {
                            n.to_string()
                        }
                    })
                    .unwrap_or_else(|_| "[number]".to_string())
            } else if value.is_boolean() {
                value.as_boolean().to_string()
            } else if value.is_null() {
                "null".to_string()
            } else if value.is_undefined() {
                "undefined".to_string()
            } else {
                value
                    .as_string()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "[object]".to_string())
            }
        })
        .collect()
}

fn first_label(arguments: &[JSValue]) -> String {
    arguments
        .first()
        .map(|value| js_values_to_strings(std::slice::from_ref(value)).remove(0))
        .unwrap_or_else(|| "default".to_string())
}

#[callback]
fn host_console_log(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::log(&js_values_to_strings(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_info(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::info(&js_values_to_strings(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_warn(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::warn(&js_values_to_strings(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_error(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::error(&js_values_to_strings(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_debug(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::debug(&js_values_to_strings(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_trace(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::trace(&js_values_to_strings(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_assert(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    let condition = arguments
        .first()
        .map(|value| value.as_boolean())
        .unwrap_or(false);
    let args = if arguments.len() > 1 {
        js_values_to_strings(&arguments[1..])
    } else {
        Vec::new()
    };
    console::assert(condition, &args);
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_clear(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    _arguments: &[JSValue],
) -> HostResult {
    console::clear();
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_count(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::count(&first_label(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_count_reset(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::count_reset(&first_label(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_time(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::time(&first_label(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_time_log(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::time_log(&first_label(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_time_end(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::time_end(&first_label(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_group(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::group(&first_label(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_group_collapsed(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::group_collapsed(&first_label(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_console_group_end(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    _arguments: &[JSValue],
) -> HostResult {
    console::group_end();
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_print(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::print(&js_values_to_strings(arguments));
    Ok(JSValue::undefined(&ctx))
}

#[callback]
fn host_println(
    ctx: JSContext,
    _function: JSObject,
    _this: JSObject,
    arguments: &[JSValue],
) -> HostResult {
    console::println(&js_values_to_strings(arguments));
    Ok(JSValue::undefined(&ctx))
}

fn install_global_function(
    ctx: &JSContext,
    name: &str,
    callback: rust_jsc::internal::JSObjectCallAsFunctionCallback,
) -> Result<(), JSError> {
    let function = JSFunction::callback(ctx, Some(name), callback);
    ctx.global_object()
        .set_property(name, &function, PropertyDescriptor::default())
        .map_err(|err| JSError::internal(err.to_string()))
}

fn install_jstd(ctx: &JSContext) -> Result<(), JSError> {
    install_global_function(ctx, "__jstd_console_log", Some(host_console_log))?;
    install_global_function(ctx, "__jstd_console_info", Some(host_console_info))?;
    install_global_function(ctx, "__jstd_console_warn", Some(host_console_warn))?;
    install_global_function(ctx, "__jstd_console_error", Some(host_console_error))?;
    install_global_function(ctx, "__jstd_console_debug", Some(host_console_debug))?;
    install_global_function(ctx, "__jstd_console_trace", Some(host_console_trace))?;
    install_global_function(ctx, "__jstd_console_assert", Some(host_console_assert))?;
    install_global_function(ctx, "__jstd_console_clear", Some(host_console_clear))?;
    install_global_function(ctx, "__jstd_console_count", Some(host_console_count))?;
    install_global_function(
        ctx,
        "__jstd_console_count_reset",
        Some(host_console_count_reset),
    )?;
    install_global_function(ctx, "__jstd_console_time", Some(host_console_time))?;
    install_global_function(ctx, "__jstd_console_time_log", Some(host_console_time_log))?;
    install_global_function(ctx, "__jstd_console_time_end", Some(host_console_time_end))?;
    install_global_function(ctx, "__jstd_console_group", Some(host_console_group))?;
    install_global_function(
        ctx,
        "__jstd_console_group_collapsed",
        Some(host_console_group_collapsed),
    )?;
    install_global_function(
        ctx,
        "__jstd_console_group_end",
        Some(host_console_group_end),
    )?;
    install_global_function(ctx, "__jstd_print", Some(host_print))?;
    install_global_function(ctx, "__jstd_println", Some(host_println))?;
    ctx.evaluate_script(jstd::bootstrap_script(), Some(1))
        .map_err(|err| JSError::internal(err.to_string()).with_filename("<jstd>"))?;
    Ok(())
}

pub struct JavaScriptCore {
    ctx: JSContext,
}

impl JavaScriptCore {
    pub fn new() -> Self {
        let ctx = JSContext::new();
        install_jstd(&ctx).expect("failed to initialize jstd builtins");
        Self { ctx }
    }
}

impl Default for JavaScriptCore {
    fn default() -> Self {
        Self::new()
    }
}

impl JSEngine for JavaScriptCore {
    fn run(&self, code: &str, _filename: &str) -> Result<JSResult, JSError> {
        let filename = _filename;
        let value = self.ctx.evaluate_script(code, Some(1)).map_err(|err| {
            JSError::runtime(err.to_string())
                .with_filename(filename)
                .with_source(code)
        })?;

        if value.is_string() {
            return Ok(JSResult::String(
                value
                    .as_string()
                    .map(|s| s.to_string())
                    .map_err(|err| JSError::runtime(err.to_string()).with_filename(filename))?,
            ));
        }

        if value.is_number() {
            let number = value
                .as_number()
                .map_err(|err| JSError::runtime(err.to_string()).with_filename(filename))?;
            if number.fract() == 0.0 {
                return Ok(JSResult::String((number as i64).to_string()));
            }
            return Ok(JSResult::String(number.to_string()));
        }

        if value.is_boolean() {
            return Ok(JSResult::String(value.as_boolean().to_string()));
        }

        if value.is_undefined() {
            return Ok(JSResult::String("undefined".to_string()));
        }

        if value.is_null() {
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
        let engine = JavaScriptCore::new();
        let result = engine.run("40 + 2", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "42"),
        }
    }

    #[test]
    fn test_run_string_literal() {
        let engine = JavaScriptCore::new();
        let result = engine.run("'hello'", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "hello"),
        }
    }

    #[test]
    fn test_run_boolean() {
        let engine = JavaScriptCore::new();
        let result = engine.run("true", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "true"),
        }
    }

    #[test]
    fn test_console_log_builtin() {
        let engine = JavaScriptCore::new();
        let result = engine.run("console.log('hello'); 42", "<test>").unwrap();
        match result {
            JSResult::String(s) => assert_eq!(s, "42"),
        }
    }
}

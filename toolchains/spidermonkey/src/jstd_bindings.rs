#![allow(dead_code)]
#![allow(unused)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::CString;
use std::os::raw::c_char;

use mozjs::jsapi::{
    JS_DefineProperty3, JS_EncodeStringToASCII, JS_NewPlainObject, JS_NewStringCopyN,
    JS_ReportErrorASCII, JSContext, JSFunctionSpec, JSNativeWrapper, JSObject, JSPROP_ENUMERATE,
    JSPropertySpec_Name, Value,
};
// don't import mozjs::rust::Handle here to avoid confusion with jsapi Handle
use mozjs::JS_ARGV;
use mozjs::context::JSContext as MozJSContext;
use mozjs::rooted;

use jstd::buffer::Buffer;
use jstd::console;
use jstd::fs;
use jstd::path;
use jstd::process;

macro_rules! js_fn {
    ($name:ident, $body:expr) => {
        unsafe extern "C" fn $name(cx: *mut JSContext, argc: u32, vp: *mut Value) -> bool {
            let result = $body(cx, argc, vp);
            match result {
                Ok(val) => {
                    unsafe {
                        *vp = val;
                    }
                    true
                }
                Err(msg) => {
                    let c_msg =
                        CString::new(msg).unwrap_or_else(|_| CString::new("error").unwrap());
                    unsafe {
                        JS_ReportErrorASCII(cx, c_msg.as_ptr());
                    }
                    false
                }
            }
        }
    };
}

fn get_arg_string(cx: *mut JSContext, vp: *mut Value, index: usize) -> Option<String> {
    unsafe {
        let args = JS_ARGV(cx, vp);
        let val = *args.add(index);
        if val.is_string() {
            let s = val.to_string();
            let encoded = JS_EncodeStringToASCII(cx, s);
            if encoded == 0 {
                return None;
            }
            let cstr = std::ffi::CStr::from_ptr(encoded as *const c_char);
            let result = cstr.to_string_lossy().to_string();
            // JS_EncodeStringToASCII returns a string that must be freed?
            // In some mozjs versions it does.
            // But let's focus on the crash first.
            Some(result)
        } else if val.is_int32() {
            Some(val.to_int32().to_string())
        } else if val.is_double() {
            Some(val.to_double().to_string())
        } else if val.is_boolean() {
            Some(val.to_boolean().to_string())
        } else {
            None
        }
    }
}

fn to_js_string(cx: *mut JSContext, s: &str) -> Value {
    unsafe {
        let js_str = JS_NewStringCopyN(cx, s.as_ptr() as *const c_char, s.len());
        if js_str.is_null() {
            mozjs::jsval::UndefinedValue()
        } else {
            mozjs::jsval::StringValue(&*js_str)
        }
    }
}

fn to_js_int32(v: i32) -> Value {
    mozjs::jsval::Int32Value(v)
}

fn to_js_bool(v: bool) -> Value {
    mozjs::jsval::BooleanValue(v)
}

js_fn!(js_path_basename, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("path.basename requires at least 1 argument".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    let ext = if argc > 1 {
        get_arg_string(cx, vp, 1)
    } else {
        None
    };
    let result = path::basename(&p, ext.as_deref());
    Ok(to_js_string(cx, &result))
});

js_fn!(js_path_dirname, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("path.dirname requires at least 1 argument".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    let result = path::dirname(&p);
    Ok(to_js_string(cx, &result))
});

js_fn!(js_path_extname, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("path.extname requires at least 1 argument".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    let result = path::extname(&p);
    Ok(to_js_string(cx, &result))
});

js_fn!(js_path_join, |cx, argc, vp| -> Result<Value, String> {
    let mut paths = Vec::new();
    for i in 0..argc as usize {
        if let Some(s) = get_arg_string(cx, vp, i) {
            paths.push(s);
        }
    }
    if paths.is_empty() {
        return Ok(to_js_string(cx, "."));
    }
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    let result = path::join(&paths[0], &path_refs[1..]);
    Ok(to_js_string(cx, &result))
});

js_fn!(js_path_resolve, |cx, argc, vp| -> Result<Value, String> {
    let mut paths = Vec::new();
    for i in 0..argc as usize {
        if let Some(s) = get_arg_string(cx, vp, i) {
            paths.push(s);
        }
    }
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    let result = path::resolve(&path_refs);
    Ok(to_js_string(cx, &result))
});

js_fn!(js_path_normalize, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("path.normalize requires at least 1 argument".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    let result = path::normalize(&p);
    Ok(to_js_string(cx, &result))
});

js_fn!(js_path_is_absolute, |cx,
                             argc,
                             vp|
 -> Result<Value, String> {
    if argc < 1 {
        return Err("path.isAbsolute requires at least 1 argument".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    let result = path::is_absolute(&p);
    Ok(to_js_bool(result))
});

js_fn!(js_process_platform, |cx,
                             _argc,
                             _vp|
 -> Result<Value, String> {
    Ok(to_js_string(cx, process::platform()))
});

js_fn!(js_process_arch, |cx, _argc, _vp| -> Result<Value, String> {
    Ok(to_js_string(cx, process::arch()))
});

js_fn!(js_process_pid, |cx, _argc, _vp| -> Result<Value, String> {
    Ok(to_js_int32(process::pid() as i32))
});

js_fn!(js_process_cwd, |cx, _argc, _vp| -> Result<Value, String> {
    Ok(to_js_string(cx, &process::cwd()))
});

js_fn!(js_process_version, |cx,
                            _argc,
                            _vp|
 -> Result<Value, String> {
    Ok(to_js_string(cx, process::version()))
});

js_fn!(js_process_env, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("process.env requires a key".to_string());
    }
    let key = get_arg_string(cx, vp, 0).unwrap_or_default();
    match process::get_env(&key) {
        Some(val) => Ok(to_js_string(cx, &val)),
        None => Ok(mozjs::jsval::UndefinedValue()),
    }
});

js_fn!(js_fs_read_file, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("fs.readFileSync requires a path".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    match fs::read_file_sync(&p) {
        Ok(content) => Ok(to_js_string(cx, &content)),
        Err(e) => Err(format!("{}", e)),
    }
});

js_fn!(js_fs_write_file, |cx, argc, vp| -> Result<Value, String> {
    if argc < 2 {
        return Err("fs.writeFileSync requires path and data".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    let data = get_arg_string(cx, vp, 1).unwrap_or_default();
    match fs::write_file_sync(&p, &data) {
        Ok(()) => Ok(mozjs::jsval::UndefinedValue()),
        Err(e) => Err(format!("{}", e)),
    }
});

js_fn!(js_fs_exists, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("fs.existsSync requires a path".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    Ok(to_js_bool(fs::exists_sync(&p)))
});

js_fn!(js_fs_mkdir, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("fs.mkdirSync requires a path".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    let recursive = if argc > 1 {
        get_arg_string(cx, vp, 1)
            .map(|s| s == "true")
            .unwrap_or(false)
    } else {
        false
    };
    match fs::mkdir_sync(&p, recursive) {
        Ok(()) => Ok(mozjs::jsval::UndefinedValue()),
        Err(e) => Err(format!("{}", e)),
    }
});

js_fn!(js_fs_unlink, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("fs.unlinkSync requires a path".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    match fs::unlink_sync(&p) {
        Ok(()) => Ok(mozjs::jsval::UndefinedValue()),
        Err(e) => Err(format!("{}", e)),
    }
});

js_fn!(js_fs_readdir, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("fs.readdirSync requires a path".to_string());
    }
    let p = get_arg_string(cx, vp, 0).unwrap_or_default();
    match fs::readdir_sync(&p) {
        Ok(entries) => {
            let result = entries.join(",");
            Ok(to_js_string(cx, &result))
        }
        Err(e) => Err(format!("{}", e)),
    }
});

js_fn!(js_buffer_from, |cx, argc, vp| -> Result<Value, String> {
    if argc < 1 {
        return Err("Buffer.from requires a string".to_string());
    }
    let s = get_arg_string(cx, vp, 0).unwrap_or_default();
    let encoding = if argc > 1 {
        get_arg_string(cx, vp, 1)
    } else {
        None
    };
    let buf = Buffer::from_string(&s, encoding.as_deref());
    let len = buf.len();
    Ok(to_js_int32(len as i32))
});

fn get_all_args_as_strings(cx: *mut JSContext, vp: *mut Value, argc: u32) -> Vec<String> {
    let mut args = Vec::new();
    for i in 0..argc as usize {
        if let Some(s) = get_arg_string(cx, vp, i) {
            args.push(s);
        } else {
            args.push("[object]".to_string());
        }
    }
    args
}

fn get_arg_bool(cx: *mut JSContext, vp: *mut Value, index: usize) -> bool {
    unsafe {
        let args = JS_ARGV(cx, vp);
        let val = *args.add(index);
        if val.is_boolean() {
            val.to_boolean()
        } else if val.is_int32() {
            val.to_int32() != 0
        } else if val.is_double() {
            val.to_double() != 0.0
        } else {
            get_arg_string(cx, vp, index)
                .map(|s| !s.is_empty() && s != "false")
                .unwrap_or(false)
        }
    }
}

js_fn!(js_console_log, |cx, argc, vp| -> Result<Value, String> {
    let args = get_all_args_as_strings(cx, vp, argc);
    console::log(&args);
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_info, |cx, argc, vp| -> Result<Value, String> {
    console::info(&get_all_args_as_strings(cx, vp, argc));
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_warn, |cx, argc, vp| -> Result<Value, String> {
    console::warn(&get_all_args_as_strings(cx, vp, argc));
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_error, |cx, argc, vp| -> Result<Value, String> {
    console::error(&get_all_args_as_strings(cx, vp, argc));
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_debug, |cx, argc, vp| -> Result<Value, String> {
    console::debug(&get_all_args_as_strings(cx, vp, argc));
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_trace, |cx, argc, vp| -> Result<Value, String> {
    console::trace(&get_all_args_as_strings(cx, vp, argc));
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_assert, |cx, argc, vp| -> Result<Value, String> {
    let condition = if argc > 0 {
        get_arg_bool(cx, vp, 0)
    } else {
        false
    };
    let args = if argc > 1 {
        get_all_args_as_strings(cx, vp, argc)
            .into_iter()
            .skip(1)
            .collect()
    } else {
        Vec::new()
    };
    console::assert(condition, &args);
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_clear, |_cx,
                          _argc,
                          _vp|
 -> Result<Value, String> {
    console::clear();
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_count, |cx, argc, vp| -> Result<Value, String> {
    let label = if argc > 0 {
        get_arg_string(cx, vp, 0).unwrap_or_else(|| "default".to_string())
    } else {
        "default".to_string()
    };
    console::count(&label);
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_count_reset, |cx,
                                argc,
                                vp|
 -> Result<Value, String> {
    let label = if argc > 0 {
        get_arg_string(cx, vp, 0).unwrap_or_else(|| "default".to_string())
    } else {
        "default".to_string()
    };
    console::count_reset(&label);
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_time, |cx, argc, vp| -> Result<Value, String> {
    let label = if argc > 0 {
        get_arg_string(cx, vp, 0).unwrap_or_else(|| "default".to_string())
    } else {
        "default".to_string()
    };
    console::time(&label);
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_time_log, |cx,
                             argc,
                             vp|
 -> Result<Value, String> {
    let label = if argc > 0 {
        get_arg_string(cx, vp, 0).unwrap_or_else(|| "default".to_string())
    } else {
        "default".to_string()
    };
    console::time_log(&label);
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_time_end, |cx,
                             argc,
                             vp|
 -> Result<Value, String> {
    let label = if argc > 0 {
        get_arg_string(cx, vp, 0).unwrap_or_else(|| "default".to_string())
    } else {
        "default".to_string()
    };
    console::time_end(&label);
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_group, |cx, argc, vp| -> Result<Value, String> {
    let label = if argc > 0 {
        get_arg_string(cx, vp, 0).unwrap_or_default()
    } else {
        String::new()
    };
    console::group(&label);
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_group_collapsed, |cx,
                                    argc,
                                    vp|
 -> Result<Value, String> {
    let label = if argc > 0 {
        get_arg_string(cx, vp, 0).unwrap_or_default()
    } else {
        String::new()
    };
    console::group_collapsed(&label);
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_console_group_end, |_cx,
                              _argc,
                              _vp|
 -> Result<Value, String> {
    console::group_end();
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_print, |cx, argc, vp| -> Result<Value, String> {
    console::print(&get_all_args_as_strings(cx, vp, argc));
    Ok(mozjs::jsval::UndefinedValue())
});

js_fn!(js_println, |cx, argc, vp| -> Result<Value, String> {
    console::println(&get_all_args_as_strings(cx, vp, argc));
    Ok(mozjs::jsval::UndefinedValue())
});

unsafe fn define_function(
    cx: *mut JSContext,
    global: mozjs::rust::Handle<*mut JSObject>,
    name: &str,
    func: unsafe extern "C" fn(*mut JSContext, u32, *mut Value) -> bool,
) {
    let name = CString::new(name).unwrap();
    mozjs::jsapi::JS_DefineFunction(
        cx,
        global.into(),
        name.as_ptr(),
        Some(func),
        0,
        JSPROP_ENUMERATE as u32,
    );
}

pub fn define_jstd_for_global(cx: &mut MozJSContext, global: mozjs::rust::Handle<*mut JSObject>) {
    unsafe {
        let cx_raw = cx.raw_cx();
        define_function(cx_raw, global, "__jstd_console_log", js_console_log);
        define_function(cx_raw, global, "__jstd_console_info", js_console_info);
        define_function(cx_raw, global, "__jstd_console_warn", js_console_warn);
        define_function(cx_raw, global, "__jstd_console_error", js_console_error);
        define_function(cx_raw, global, "__jstd_console_debug", js_console_debug);
        define_function(cx_raw, global, "__jstd_console_trace", js_console_trace);
        define_function(cx_raw, global, "__jstd_console_assert", js_console_assert);
        define_function(cx_raw, global, "__jstd_console_clear", js_console_clear);
        define_function(cx_raw, global, "__jstd_console_count", js_console_count);
        define_function(
            cx_raw,
            global,
            "__jstd_console_count_reset",
            js_console_count_reset,
        );
        define_function(cx_raw, global, "__jstd_console_time", js_console_time);
        define_function(
            cx_raw,
            global,
            "__jstd_console_time_log",
            js_console_time_log,
        );
        define_function(
            cx_raw,
            global,
            "__jstd_console_time_end",
            js_console_time_end,
        );
        define_function(cx_raw, global, "__jstd_console_group", js_console_group);
        define_function(
            cx_raw,
            global,
            "__jstd_console_group_collapsed",
            js_console_group_collapsed,
        );
        define_function(
            cx_raw,
            global,
            "__jstd_console_group_end",
            js_console_group_end,
        );
        define_function(cx_raw, global, "__jstd_print", js_print);
        define_function(cx_raw, global, "__jstd_println", js_println);
    }
}

pub fn define_jstd(_cx: &mut mozjs::context::JSContext) {}

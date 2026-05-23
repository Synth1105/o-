use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

struct ConsoleState {
    counts: HashMap<String, u64>,
    timers: HashMap<String, Instant>,
    group_stack: Vec<String>,
    group_collapsed: Vec<String>,
}

impl Default for ConsoleState {
    fn default() -> Self {
        ConsoleState {
            counts: HashMap::new(),
            timers: HashMap::new(),
            group_stack: Vec::new(),
            group_collapsed: Vec::new(),
        }
    }
}

static CONSOLE_STATE: LazyLock<Mutex<ConsoleState>> =
    LazyLock::new(|| Mutex::new(ConsoleState::default()));

fn get_indentation() -> String {
    let state = CONSOLE_STATE.lock().unwrap();
    "  ".repeat(state.group_stack.len())
}

fn print_with_prefix(prefix: &str, args: &[String]) {
    let indent = get_indentation();
    let message = args.join(" ");
    let output = format!("{}{} {}", indent, prefix, message);
    let _ = writeln!(io::stdout(), "{}", output.trim());
    let _ = io::stdout().flush();
}

fn print_plain(args: &[String]) {
    let indent = get_indentation();
    let message = args.join(" ");
    let output = format!("{}{}", indent, message);
    let _ = writeln!(io::stdout(), "{}", output.trim());
    let _ = io::stdout().flush();
}

pub fn log(args: &[String]) {
    print_plain(args);
}

pub fn info(args: &[String]) {
    print_with_prefix("ℹ️", args);
}

pub fn warn(args: &[String]) {
    print_with_prefix("⚠️", args);
}

pub fn error(args: &[String]) {
    print_with_prefix("❌", args);
}

pub fn debug(args: &[String]) {
    print_with_prefix("🐛", args);
}

pub fn trace(args: &[String]) {
    let indent = get_indentation();
    println!("{}Trace:", indent);
    for (i, arg) in args.iter().enumerate() {
        println!("{}  {}: {}", indent, i, arg);
    }
    let _ = io::stdout().flush();
}

pub fn assert(condition: bool, args: &[String]) {
    if !condition {
        let message = if args.is_empty() {
            vec!["Assertion failed".to_string()]
        } else {
            args.to_vec()
        };
        print_with_prefix("❌", &message);
    }
}

pub fn clear() {
    #[cfg(unix)]
    {
        print!("\x1B[2J\x1B[1;1H");
    }
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "cls"])
            .status();
    }
    let _ = io::stdout().flush();
}

pub fn count(label: &str) {
    let mut state = CONSOLE_STATE.lock().unwrap();
    let counter = state.counts.entry(label.to_string()).or_insert(0);
    *counter += 1;
    let count = *counter;
    drop(state);
    println!("{}: {}", label, count);
    let _ = io::stdout().flush();
}

pub fn count_reset(label: &str) {
    let mut state = CONSOLE_STATE.lock().unwrap();
    state.counts.remove(label);
    println!("{}: 0", label);
    let _ = io::stdout().flush();
}

pub fn count_all_reset() {
    let mut state = CONSOLE_STATE.lock().unwrap();
    state.counts.clear();
    let _ = io::stdout().flush();
}

pub fn time(label: &str) {
    let mut state = CONSOLE_STATE.lock().unwrap();
    state.timers.insert(label.to_string(), Instant::now());
    println!("{}: timer started", label);
    let _ = io::stdout().flush();
}

pub fn time_log(label: &str) {
    let state = CONSOLE_STATE.lock().unwrap();
    if let Some(start) = state.timers.get(label) {
        let elapsed = start.elapsed();
        println!("{}: {}ms", label, elapsed.as_millis());
    } else {
        println!("Timer '{}' does not exist", label);
    }
    let _ = io::stdout().flush();
}

pub fn time_end(label: &str) {
    let mut state = CONSOLE_STATE.lock().unwrap();
    if let Some(start) = state.timers.remove(label) {
        let elapsed = start.elapsed();
        println!("{}: {}ms - timer ended", label, elapsed.as_millis());
    } else {
        println!("Timer '{}' does not exist", label);
    }
    let _ = io::stdout().flush();
}

pub fn group(label: &str) {
    println!("{}{}", get_indentation(), label);
    let mut state = CONSOLE_STATE.lock().unwrap();
    state.group_stack.push(label.to_string());
    let _ = io::stdout().flush();
}

pub fn group_collapsed(label: &str) {
    println!("{}{} (collapsed)", get_indentation(), label);
    let mut state = CONSOLE_STATE.lock().unwrap();
    state.group_stack.push(label.to_string());
    state.group_collapsed.push(label.to_string());
    let _ = io::stdout().flush();
}

pub fn group_end() {
    let mut state = CONSOLE_STATE.lock().unwrap();
    if state.group_stack.pop().is_none() {
        println!("No group to end");
    }
    let _ = io::stdout().flush();
}

pub fn table(headers: &[String], rows: &[Vec<String>]) {
    let mut all_rows: Vec<Vec<String>> = vec![headers.to_vec()];
    all_rows.extend(rows.iter().cloned());

    let mut col_widths = vec![0; headers.len()];
    for row in &all_rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    let indent = get_indentation();
    for row in &all_rows {
        let line: String = row
            .iter()
            .enumerate()
            .map(|(i, cell)| format!("{:<width$}", cell, width = col_widths[i]))
            .collect::<Vec<_>>()
            .join(" | ");
        println!("{}{}", indent, line);
    }
    let _ = io::stdout().flush();
}

pub fn dir<T: std::fmt::Debug>(value: &T) {
    let indent = get_indentation();
    println!("{}{:#?}", indent, value);
    let _ = io::stdout().flush();
}

pub fn dirxml<T: std::fmt::Debug>(value: &T) {
    let indent = get_indentation();
    println!("{}{:#?}", indent, value);
    let _ = io::stdout().flush();
}

pub fn profile(label: &str) {
    println!("Profile '{}' started", label);
    let _ = io::stdout().flush();
}

pub fn profile_end(label: &str) {
    println!("Profile '{}' ended", label);
    let _ = io::stdout().flush();
}

pub fn time_stamp(label: &str) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    println!("{}: {}ms", label, now.as_millis());
    let _ = io::stdout().flush();
}

pub fn print(args: &[String]) {
    print_plain(args);
}

pub fn println(args: &[String]) {
    print_plain(args);
}

pub mod buffer;
pub mod console;
pub mod events;
pub mod fs;
pub mod path;
pub mod process;
pub mod timers;

pub use buffer::Buffer;
pub use console::*;
pub use events::EventEmitter;
pub use timers::TimerManager;

pub fn bootstrap_script() -> &'static str {
    r#"
(() => {
  const g = globalThis;
  const host = (name) => {
    const fn = g[name];
    return typeof fn === 'function' ? fn : undefined;
  };
  const fallback = (...args) => {
    const printer = host('__jstd_print');
    if (printer) printer(...args);
  };
  const bind = (name, alt) => host(name) || alt || fallback;

  g.console = {
    log: bind('__jstd_console_log'),
    info: bind('__jstd_console_info'),
    warn: bind('__jstd_console_warn'),
    error: bind('__jstd_console_error'),
    debug: bind('__jstd_console_debug'),
    trace: bind('__jstd_console_trace'),
    assert: bind('__jstd_console_assert'),
    clear: bind('__jstd_console_clear', () => {}),
    count: bind('__jstd_console_count'),
    countReset: bind('__jstd_console_count_reset'),
    time: bind('__jstd_console_time'),
    timeLog: bind('__jstd_console_time_log'),
    timeEnd: bind('__jstd_console_time_end'),
    group: bind('__jstd_console_group'),
    groupCollapsed: bind('__jstd_console_group_collapsed'),
    groupEnd: bind('__jstd_console_group_end', () => {}),
    table: bind('__jstd_console_log'),
    dir: bind('__jstd_console_log'),
    dirxml: bind('__jstd_console_log'),
    profile: bind('__jstd_console_log'),
    profileEnd: bind('__jstd_console_log'),
    timeStamp: bind('__jstd_console_log'),
  };

  g.print = bind('__jstd_print');
  g.println = bind('__jstd_println', g.print);
})();
"#
}

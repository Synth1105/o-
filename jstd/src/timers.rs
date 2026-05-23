use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct TimerEntry {
    id: u64,
    #[allow(dead_code)]
    handle: Option<thread::JoinHandle<()>>,
    cancelled: bool,
}

pub struct TimerManager {
    timers: Mutex<Vec<TimerEntry>>,
}

impl TimerManager {
    pub fn new() -> Self {
        TimerManager {
            timers: Mutex::new(Vec::new()),
        }
    }

    pub fn set_timeout<F>(&self, callback: F, ms: u64) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let duration = Duration::from_millis(ms);

        let handle = thread::spawn(move || {
            thread::sleep(duration);
            callback();
        });

        let mut timers = self.timers.lock().unwrap();
        timers.push(TimerEntry {
            id,
            handle: Some(handle),
            cancelled: false,
        });

        id
    }

    pub fn set_interval<F>(&self, callback: F, ms: u64) -> u64
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let duration = Duration::from_millis(ms);
        let callback = std::sync::Arc::new(callback);

        let handle = {
            let callback = callback.clone();
            thread::spawn(move || {
                loop {
                    thread::sleep(duration);
                    callback();
                }
            })
        };

        let mut timers = self.timers.lock().unwrap();
        timers.push(TimerEntry {
            id,
            handle: Some(handle),
            cancelled: false,
        });

        id
    }

    pub fn clear_timeout(&self, id: u64) {
        let mut timers = self.timers.lock().unwrap();
        for entry in timers.iter_mut() {
            if entry.id == id {
                entry.cancelled = true;
                break;
            }
        }
    }

    pub fn clear_interval(&self, id: u64) {
        self.clear_timeout(id);
    }

    pub fn clear_all(&self) {
        let mut timers = self.timers.lock().unwrap();
        for entry in timers.iter_mut() {
            entry.cancelled = true;
        }
        timers.clear();
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

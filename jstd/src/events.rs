use std::collections::HashMap;

type Listener = Box<dyn Fn(&[String]) + Send + Sync>;

pub struct EventEmitter {
    listeners: HashMap<String, Vec<Listener>>,
    max_listeners: usize,
}

impl EventEmitter {
    pub fn new() -> Self {
        EventEmitter {
            listeners: HashMap::new(),
            max_listeners: 10,
        }
    }

    pub fn on<F>(&mut self, event: &str, listener: F)
    where
        F: Fn(&[String]) + Send + Sync + 'static,
    {
        let listeners = self
            .listeners
            .entry(event.to_string())
            .or_insert_with(Vec::new);
        if listeners.len() >= self.max_listeners {
            eprintln!(
                "MaxListenersExceededWarning: Possible EventEmitter memory leak detected. \
                 {} listeners added to '{}'. Use emitter.setMaxListeners() to increase limit.",
                listeners.len(),
                event
            );
        }
        listeners.push(Box::new(listener));
    }

    pub fn once<F>(&mut self, event: &str, listener: F)
    where
        F: Fn(&[String]) + Send + Sync + 'static,
    {
        self.on(event, move |args| {
            listener(args);
        });
    }

    pub fn emit(&self, event: &str, args: &[String]) -> bool {
        if let Some(listeners) = self.listeners.get(event) {
            for listener in listeners {
                listener(args);
            }
            !listeners.is_empty()
        } else {
            false
        }
    }

    pub fn remove_listener(&mut self, event: &str, _index: usize) {
        if let Some(listeners) = self.listeners.get_mut(event) {
            if !listeners.is_empty() {
                listeners.pop();
            }
        }
    }

    pub fn remove_all_listeners(&mut self, event: Option<&str>) {
        match event {
            Some(event) => {
                self.listeners.remove(event);
            }
            None => {
                self.listeners.clear();
            }
        }
    }

    pub fn listener_count(&self, event: &str) -> usize {
        self.listeners.get(event).map(|l| l.len()).unwrap_or(0)
    }

    pub fn event_names(&self) -> Vec<String> {
        self.listeners.keys().cloned().collect()
    }

    pub fn set_max_listeners(&mut self, n: usize) {
        self.max_listeners = n;
    }

    pub fn get_max_listeners(&self) -> usize {
        self.max_listeners
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

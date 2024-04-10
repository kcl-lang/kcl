use crate::config_manager::Config;
use crate::file::{FileEvent, FileHandler, Observer};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

/// HandlerRegistry to register and manage file handlers
pub struct HandlerRegistry {
    handlers: HashMap<String, Box<dyn FileHandler>>,
}

impl HandlerRegistry {
    /// Create a new HandlerRegistry instance
    pub fn new() -> Self {
        HandlerRegistry {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for a file type
    pub fn register_handler(&mut self, file_type: &str, handler: Box<dyn FileHandler>) {
        self.handlers.insert(file_type.to_string(), handler);
    }

    /// Get a handler for a file type
    pub fn get_handler(&self, file_type: &str) -> Option<&Box<dyn FileHandler>> {
        self.handlers.get(file_type)
    }

    /// Handle file event
    pub fn handle_event(&self, event: &FileEvent) {
        match event {
            FileEvent::Modified(file) => {
                if let Some(handler) =
                    self.get_handler(&file.detect_file_type().unwrap_or_default())
                {
                    handler.handle(file);
                }
            }
        }
    }
}

/// KCL Watch System structure to manage the observer and handler registry
pub struct KCLWatchSystem {
    observer: Arc<Mutex<Observer>>,
    handler_registry: Arc<Mutex<HandlerRegistry>>,
}

impl KCLWatchSystem {
    /// Create a new KCL Watch System instance with a configuration
    pub fn new(config: Config) -> Self {
        let observer = Arc::new(Mutex::new(Observer::new(config.clone())));
        let handler_registry = Arc::new(Mutex::new(HandlerRegistry::new()));
        KCLWatchSystem {
            observer,
            handler_registry,
        }
    }

    /// Start the observer
    pub fn start_observer(&self) {
        let observer = self.observer.clone();
        let handler_registry = self.handler_registry.clone();
        thread::spawn(move || loop {
            let mut observer_lock = observer.lock().unwrap();
            let event_opt = observer_lock.iter_events().next();
            if let Some(event) = event_opt {
                handler_registry.lock().unwrap().handle_event(&event);
            }
        });
    }
}

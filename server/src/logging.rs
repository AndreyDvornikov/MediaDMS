use std::collections::VecDeque;
use std::sync::Mutex;

use chrono::Utc;

use crate::models::{Entity, RequestLogEntry};

pub struct RequestLogger {
    cap: usize,
    inner: Mutex<VecDeque<RequestLogEntry>>,
}

impl RequestLogger {
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            inner: Mutex::new(VecDeque::with_capacity(cap)),
        }
    }

    pub fn push(&self, ip: String, entity: Entity, error: u8, error_message: Option<String>) {
        let mut guard = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        if guard.len() >= self.cap {
            guard.pop_front();
        }

        guard.push_back(RequestLogEntry {
            timestamp_utc: Utc::now().to_rfc3339(),
            ip,
            entity,
            error,
            error_message,
        });
    }

    pub fn list(&self) -> Vec<RequestLogEntry> {
        match self.inner.lock() {
            Ok(g) => g.iter().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
}

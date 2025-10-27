use std::collections::VecDeque;
/// Debug logging utilities for tracking organism behavior and diagnosing issues
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

/// Global flag to enable detailed debug logging
/// Set to false to reduce console spam
pub static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Track organism lifecycle events
pub static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
pub static COPIES: AtomicU64 = AtomicU64::new(0);
pub static DIVISIONS: AtomicU64 = AtomicU64::new(0);
pub static FAILED_DIVISIONS: AtomicU64 = AtomicU64::new(0);

/// Recent events log (circular buffer)
static EVENTS: Mutex<Option<VecDeque<String>>> = Mutex::new(None);
const MAX_EVENTS: usize = 100;

/// Initialize the debug system
pub fn init() {
    let mut events = EVENTS.lock().unwrap();
    *events = Some(VecDeque::with_capacity(MAX_EVENTS));
    // Don't call log_event here - it would deadlock since we already hold the mutex
    if DEBUG_ENABLED.load(Ordering::Relaxed) {
        println!("[DEBUG] Debug system initialized");
    }
}

/// Log a debug event
pub fn log_event(msg: impl AsRef<str>) {
    if let Ok(mut events) = EVENTS.lock() {
        if let Some(queue) = events.as_mut() {
            if queue.len() >= MAX_EVENTS {
                queue.pop_front();
            }
            queue.push_back(msg.as_ref().to_string());
        }
    }

    if DEBUG_ENABLED.load(Ordering::Relaxed) {
        println!("[DEBUG] {}", msg.as_ref());
    }
}

/// Get recent events for display
pub fn get_recent_events(count: usize) -> Vec<String> {
    if let Ok(events) = EVENTS.lock() {
        if let Some(queue) = events.as_ref() {
            return queue.iter().rev().take(count).rev().cloned().collect();
        }
    }
    Vec::new()
}

/// Print statistics summary
pub fn print_stats() {
    println!("\n=== Debug Statistics ===");
    println!("Allocations: {}", ALLOCATIONS.load(Ordering::Relaxed));
    println!("Copies: {}", COPIES.load(Ordering::Relaxed));
    println!("Divisions: {}", DIVISIONS.load(Ordering::Relaxed));
    println!(
        "Failed Divisions: {}",
        FAILED_DIVISIONS.load(Ordering::Relaxed)
    );
    println!("=======================\n");
}

/// Reset all counters
pub fn reset_stats() {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    COPIES.store(0, Ordering::Relaxed);
    DIVISIONS.store(0, Ordering::Relaxed);
    FAILED_DIVISIONS.store(0, Ordering::Relaxed);
}

/// Track organism state for debugging
#[derive(Debug, Clone)]
pub struct OrganismDebugInfo {
    pub genome_size: usize,
    pub child_allocated: bool,
    pub child_size: usize,
    pub copy_progress: usize,
    pub ip: usize,
    pub read_head: usize,
    pub write_head: usize,
    pub flow_head: usize,
    pub age: u64,
}

impl OrganismDebugInfo {
    pub fn format(&self) -> String {
        format!(
            "genome:{} child:{} progress:{}/{} ip:{} rh:{} wh:{} fh:{} age:{}",
            self.genome_size,
            if self.child_allocated { "Y" } else { "N" },
            self.copy_progress,
            self.child_size,
            self.ip,
            self.read_head,
            self.write_head,
            self.flow_head,
            self.age
        )
    }
}

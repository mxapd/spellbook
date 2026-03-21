use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn init_logging() {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("spellbook.log")
        .ok();
    *LOG_FILE.lock().unwrap() = file;
}

pub fn log(level: &str, msg: &str) {
    if let Some(ref mut file) = *LOG_FILE.lock().unwrap() {
        let timestamp = chrono_lite_timestamp();
        let _ = writeln!(file, "[{}] [{}] {}", timestamp, level, msg);
        let _ = file.flush();
    }
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:09}", duration.as_secs(), duration.subsec_nanos())
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::logging::log("INFO", &format!($($arg)*)); };
}
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => { $crate::logging::log("DEBUG", &format!($($arg)*)); };
}
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::logging::log("ERROR", &format!($($arg)*)); };
}
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::logging::log("WARN", &format!($($arg)*)); };
}

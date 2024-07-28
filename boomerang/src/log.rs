use ic_canister_log::{declare_log_buffer, export as export_logs, GlobalBuffer, Sink};
use serde::Deserialize;

declare_log_buffer!(name = INFO_BUF, capacity = 1000);

pub const INFO: PrintProxySink = PrintProxySink("INFO", &INFO_BUF);

pub struct PrintProxySink(&'static str, &'static GlobalBuffer);

impl Sink for PrintProxySink {
    fn append(&self, entry: ic_canister_log::LogEntry) {
        ic_cdk::println!("{} {}:{} {}", self.0, entry.file, entry.line, entry.message);
        self.1.append(entry)
    }
}

#[derive(Clone, serde::Serialize, Deserialize, Debug)]
pub struct LogEntry {
    pub timestamp: u64,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub counter: u64,
}

#[derive(Clone, Default, serde::Serialize, Deserialize, Debug)]
pub struct Log {
    pub entries: Vec<LogEntry>,
}

impl Log {
    pub fn push_logs(&mut self) {
        for entry in export_logs(&INFO_BUF) {
            self.entries.push(LogEntry {
                timestamp: entry.timestamp,
                counter: entry.counter,
                file: entry.file.to_string(),
                line: entry.line,
                message: entry.message,
            });
        }
    }

    pub fn serialize_logs(&self, max_body_size: usize) -> String {
        let mut entries_json: String = serde_json::to_string(&self).unwrap_or_default();

        if entries_json.len() > max_body_size {
            let mut left = 0;
            let mut right = self.entries.len();

            while left < right {
                let mid = left + (right - left) / 2;
                let mut temp_log = self.clone();
                temp_log.entries.truncate(mid);
                let temp_entries_json = serde_json::to_string(&temp_log).unwrap_or_default();

                if temp_entries_json.len() <= max_body_size {
                    entries_json = temp_entries_json;
                    left = mid + 1;
                } else {
                    right = mid;
                }
            }
        }
        entries_json
    }
}

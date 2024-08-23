use serde::{Deserialize, Serialize};

// Event data that is sent from clients and persisted in db as json.
// To preserve backwards compatibility with any client version,
// only new, optional fields should be added to these structures

#[derive(Serialize, Deserialize, Debug)]
pub struct EventFileLocation {
    #[serde(rename = "f")]
    file: String,
    #[serde(rename = "l")]
    line: u32,
    #[serde(rename = "c")]
    column: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEvent {
    #[serde(rename = "ts")]
    timestamp: u64,
    #[serde(rename = "lvl")]
    level: u8,
    #[serde(rename = "msg")]
    message: String,
    #[serde(rename = "mod")]
    module: Option<String>,
    #[serde(rename = "f")]
    file: Option<String>,
    #[serde(rename = "l")]
    line: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventData {
    #[serde(rename = "loc")]
    location: Option<EventFileLocation>,
    #[serde(rename = "ver")]
    version: String,
    os: String,
    arch: String,
    #[serde(rename = "tid")]
    thread_id: String,
    #[serde(rename = "tname")]
    thread_name: Option<String>,
    #[serde(rename = "trace")]
    backtrace: String,
    #[serde(rename = "log")]
    log_messages: Vec<LogEvent>,
}

use serde::{Deserialize, Serialize};

// Event data that is sent from clients and persisted in db as json.
// To preserve backwards compatibility with any client version,
// only new, optional fields should be added to these structures

#[derive(Serialize, Deserialize, Debug)]
pub struct EventFileLocation {
    #[serde(rename = "f")]
    pub file: String,
    #[serde(rename = "l")]
    pub line: u32,
    #[serde(rename = "c")]
    pub column: Option<u32>,
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
    pub title: String,
    #[serde(rename = "loc")]
    pub location: Option<EventFileLocation>,
    #[serde(rename = "ver")]
    pub version: Option<String>,
    pub os: String,
    pub arch: String,
    #[serde(rename = "tid")]
    pub thread_id: Option<String>,
    #[serde(rename = "tname")]
    pub thread_name: Option<String>,
    #[serde(rename = "trace")]
    pub backtrace: String,
    #[serde(rename = "log")]
    pub log_messages: Vec<LogEvent>,
}

impl EventData {
    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn example() -> Self {
        Self {
            title: "called `Option::unwrap()` on a `None` value".into(),
            location: Some(EventFileLocation {
                file: "stc/main.rs".into(),
                line: 45,
                column: Some(12),
            }),
            version: Some("1.2.3".into()),
            os: "linux".into(),
            arch: "x86_64".into(),
            thread_id: Some("ThreadId(1)".into()),
            thread_name: Some("main".into()),
            backtrace: r#"stack backtrace:
0: playground::main::h6849180917e9510b (0x55baf1676201)
            at src/main.rs:4
1: std::rt::lang_start::{{closure}}::hb3ceb20351fe39ee (0x55baf1675faf)
            at /rustc/3c235d5600393dfe6c36eeed34042efad8d4f26e/src/libstd/rt.rs:64
2: {{closure}} (0x55baf16be492)
            at src/libstd/rt.rs:49
    do_call<closure,i32>
            at src/libstd/panicking.rs:293
3: __rust_maybe_catch_panic (0x55baf16c00b9)
            at src/libpanic_unwind/lib.rs:87
4: try<i32,closure> (0x55baf16bef9c)
            at src/libstd/panicking.rs:272
    catch_unwind<closure,i32>
            at src/libstd/panic.rs:388
    lang_start_internal
            at src/libstd/rt.rs:48
5: std::rt::lang_start::h2c4217f9057b6ddb (0x55baf1675f88)
            at /rustc/3c235d5600393dfe6c36eeed34042efad8d4f26e/src/libstd/rt.rs:64
6: main (0x55baf16762f9)
7: __libc_start_main (0x7fab051b9b96)
8: _start (0x55baf1675e59)
9: <unknown> (0x0)"#
                .into(),
            log_messages: vec![LogEvent {
                timestamp: chrono::Utc::now().timestamp() as u64,
                level: 1,
                message: "Error message".into(),
                module: Some("my_module".into()),
                file: Some("src/main.rs".into()),
                line: Some(42),
            }],
        }
    }
}

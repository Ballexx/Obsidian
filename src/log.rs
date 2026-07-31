use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}
impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
}

pub struct LogEntry {
    log_level: LogLevel,
    message: String,
    timestamp: SystemTime,
}

impl LogEntry {
    pub fn new() -> Self {
        LogEntry {
            log_level: LogLevel::Info,
            message: "No message provided".to_owned(),
            timestamp: SystemTime::now(),
        }
    }

    pub fn set_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        return self;
    }

    pub fn set_message(mut self, msg: impl Into<String>) -> Self {
        self.message = msg.into();
        return self;
    }
}

#[macro_export]
macro_rules! log_err {
    ($level:expr, $message:expr) => {
        LogEntry::new().set_level($level).set_message($message);
    };
}

#[macro_export]
macro_rules! respond_and_return {
    ($status_code:expr, $response:expr, $socket:expr) => {
        $response.set_status($status_code);
        $response.send($socket);
        return;
    };
}

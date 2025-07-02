use {
    chrono::Timelike,
    broccli::{
        Color, TextStyle
    },
    log::{
        Log, Metadata, 
        Record, Level,
        SetLoggerError
    },
    crate::{
        io::{
            stdout, Write
        },
    }
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LogInfo {
    Time,
    Level,
    Target,
    Module,
    Message,
}

#[derive(Clone, Debug)]
pub struct LogPlan {
    components: Vec<LogInfo>,
    separator: String,
}

impl LogPlan {
    pub fn new(components: Vec<LogInfo>) -> Self {
        Self {
            components,
            separator: " ".to_string(),
        }
    }

    pub fn with_separator(mut self, separator: String) -> Self {
        self.separator = separator;
        self
    }

    pub fn default() -> Self {
        Self::new(vec![
            LogInfo::Level,
            LogInfo::Time,
            LogInfo::Target,
            LogInfo::Message,
        ])
    }

    pub fn simple() -> Self {
        Self::new(vec![
            LogInfo::Level,
            LogInfo::Message,
        ])
    }

    pub fn detailed() -> Self {
        Self::new(vec![
            LogInfo::Time,
            LogInfo::Level,
            LogInfo::Module,
            LogInfo::Target,
            LogInfo::Message,
        ])
    }
}

pub struct Logger {
    level: Level,
    plan: LogPlan,
}

impl Logger {
    pub fn new(level: Level, plan: LogPlan) -> Self {
        Self {
            level,
            plan,
        }
    }

    pub fn init(self) -> Result<(), SetLoggerError> {
        let level = self.level;

        // Convert to static reference using Box::leak
        let logger: &'static Logger = Box::leak(Box::new(self));
        log::set_logger(logger)?;
        log::set_max_level(level.to_level_filter());
        Ok(())
    }

    fn format_level(&self, level: Level) -> String {
        match level {
            Level::Error => "ERROR".to_string(),
            Level::Warn => "WARN ".to_string(),
            Level::Info => "INFO ".to_string(),
            Level::Debug => "DEBUG".to_string(),
            Level::Trace => "TRACE".to_string(),
        }
    }

    fn get_level_color(&self, level: Level) -> Color {
        match level {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Info => Color::Green,
            Level::Debug => Color::Cyan,
            Level::Trace => Color::White,
        }
    }

    fn format_component(&self, component: LogInfo, record: &Record) -> String {
        match component {
            LogInfo::Time => {
                let time = chrono::Local::now().time();
                
                format!("[{:02}:{:02}:{:02}]", time.hour(), time.minute(), time.second())
            },
            LogInfo::Level => {
                format!("{}", self.format_level(record.level()).colorize(self.get_level_color(record.level())))
            },
            LogInfo::Target => record.target().to_string(),
            LogInfo::Module => record.module_path().unwrap_or("unknown").to_string(),
            LogInfo::Message => format!("- {}", record.args()),
        }
    }

    fn format_log(&self, record: &Record) -> String {
        let mut parts = Vec::new();

        for component in &self.plan.components {
            let formatted = self.format_component(*component, record);
            if !formatted.is_empty() {
                parts.push(formatted);
            }
        }

        parts.join(&self.plan.separator)
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let formatted_log = self.format_log(record);
            println!("{}", formatted_log);
            stdout().flush().unwrap_or(());
        }
    }

    fn flush(&self) {
        stdout().flush().unwrap_or(());
    }
}
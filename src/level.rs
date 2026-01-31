use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
            Level::Fatal => "FATAL",
        }
    }

    pub fn from_severity(severity: i32) -> Option<Self> {
        match severity {
            1..=4 => Some(Level::Trace),
            5..=8 => Some(Level::Debug),
            9..=12 => Some(Level::Info),
            13..=16 => Some(Level::Warn),
            17..=20 => Some(Level::Error),
            21..=24 => Some(Level::Fatal),
            _ => None,
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "TRACE" => Ok(Level::Trace),
            "DEBUG" => Ok(Level::Debug),
            "INFO" => Ok(Level::Info),
            "WARN" | "WARNING" => Ok(Level::Warn),
            "ERROR" => Ok(Level::Error),
            "FATAL" => Ok(Level::Fatal),
            _ => Err(format!("Invalid level: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_from_severity() {
        assert_eq!(Level::from_severity(2), Some(Level::Trace));
        assert_eq!(Level::from_severity(5), Some(Level::Debug));
        assert_eq!(Level::from_severity(9), Some(Level::Info));
        assert_eq!(Level::from_severity(13), Some(Level::Warn));
        assert_eq!(Level::from_severity(17), Some(Level::Error));
        assert_eq!(Level::from_severity(21), Some(Level::Fatal));
        assert_eq!(Level::from_severity(0), None);
        assert_eq!(Level::from_severity(100), None);
    }

    #[test]
    fn test_level_from_str() {
        assert_eq!("TRACE".parse::<Level>().unwrap(), Level::Trace);
        assert_eq!("DEBUG".parse::<Level>().unwrap(), Level::Debug);
        assert_eq!("INFO".parse::<Level>().unwrap(), Level::Info);
        assert_eq!("WARN".parse::<Level>().unwrap(), Level::Warn);
        assert_eq!("WARNING".parse::<Level>().unwrap(), Level::Warn);
        assert_eq!("ERROR".parse::<Level>().unwrap(), Level::Error);
        assert_eq!("FATAL".parse::<Level>().unwrap(), Level::Fatal);
        assert_eq!("info".parse::<Level>().unwrap(), Level::Info);
        assert!("INVALID".parse::<Level>().is_err());
    }

    #[test]
    fn test_level_display() {
        assert_eq!(Level::Info.to_string(), "INFO");
        assert_eq!(Level::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_level_ordering() {
        assert!(Level::Trace < Level::Debug);
        assert!(Level::Debug < Level::Info);
        assert!(Level::Info < Level::Warn);
        assert!(Level::Warn < Level::Error);
        assert!(Level::Error < Level::Fatal);
    }
}

use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticKind {
    Parse,
    Validation,
    Io,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
    pub source_path: Option<String>,
    pub rule_id: Option<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn parse(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Parse,
            message: message.into(),
            source_path: None,
            rule_id: None,
            help: None,
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Validation,
            message: message.into(),
            source_path: None,
            rule_id: None,
            help: None,
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::Io,
            message: message.into(),
            source_path: None,
            rule_id: None,
            help: None,
        }
    }

    pub fn with_source_path(mut self, source_path: impl Into<String>) -> Self {
        self.source_path = Some(source_path.into());
        self
    }

    pub fn with_rule_id(mut self, rule_id: impl Into<String>) -> Self {
        self.rule_id = Some(rule_id.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)?;
        if let Some(source_path) = &self.source_path {
            write!(f, " [file: {source_path}]")?;
        }
        if let Some(rule_id) = &self.rule_id {
            write!(f, " [rule: {rule_id}]")?;
        }
        if let Some(help) = &self.help {
            write!(f, "\nhelp: {help}")?;
        }
        Ok(())
    }
}

impl Error for Diagnostic {}

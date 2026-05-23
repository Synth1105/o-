use thiserror::Error;

pub enum JSResult {
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JSErrorKind {
    Execution,
    Compile,
    Runtime,
    Internal,
}

#[derive(Error, Debug)]
pub struct JSError {
    kind: JSErrorKind,
    msg: String,
    filename: Option<String>,
    snippet: Option<String>,
}

impl JSError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            kind: JSErrorKind::Execution,
            msg: msg.into(),
            filename: None,
            snippet: None,
        }
    }

    pub fn compile(msg: impl Into<String>) -> Self {
        Self::new(msg).with_kind(JSErrorKind::Compile)
    }

    pub fn runtime(msg: impl Into<String>) -> Self {
        Self::new(msg).with_kind(JSErrorKind::Runtime)
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(msg).with_kind(JSErrorKind::Internal)
    }

    pub fn with_kind(mut self, kind: JSErrorKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.snippet = Some(source.into());
        self
    }

    pub fn kind(&self) -> JSErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.msg
    }

    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    pub fn source_snippet(&self) -> Option<&str> {
        self.snippet.as_deref()
    }
}

impl std::fmt::Display for JSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = use_color();
        let red = ansi(color, "1;31");
        let yellow = ansi(color, "1;33");
        let cyan = ansi(color, "1;36");
        let dim = ansi(color, "2");
        let reset = ansi(color, "0");

        let kind = match self.kind {
            JSErrorKind::Execution => "Execution Error",
            JSErrorKind::Compile => "Compile Error",
            JSErrorKind::Runtime => "Runtime Error",
            JSErrorKind::Internal => "Internal Error",
        };

        write!(f, "{red}{kind}{reset}: {}", self.msg)?;

        if let Some(filename) = &self.filename {
            write!(f, "\n{cyan}--> {reset}{filename}")?;
        }

        if let Some(source) = self
            .snippet
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let snippet = shorten(source, 140);
            write!(f, "\n{dim} |{reset} {yellow}{snippet}{reset}")?;
        }

        Ok(())
    }
}

fn use_color() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    !matches!(std::env::var("TERM").ok().as_deref(), Some("dumb"))
}

fn ansi(enabled: bool, code: &str) -> &'static str {
    if !enabled {
        return "";
    }
    match code {
        "0" => "\x1b[0m",
        "1;31" => "\x1b[1;31m",
        "1;33" => "\x1b[1;33m",
        "1;36" => "\x1b[1;36m",
        "2" => "\x1b[2m",
        _ => "",
    }
}

fn shorten(input: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for ch in input.chars().take(max_chars) {
        if ch == '\n' || ch == '\r' {
            out.push(' ');
        } else {
            out.push(ch);
        }
    }
    if input.chars().count() > max_chars {
        out.push_str("...");
    }
    out
}

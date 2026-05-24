use std::io::{self, BufRead, IsTerminal, Read, Write};

use serde_json::Value;

use crate::kernel::{Frame, FrameData, HostError, HostResult, Syscall, SyscallInfo};

/// Built-in console HAL syscall handler.
///
/// TARGET: `stdlib/norma/hal/consolum.fab` is the first host-effect contract
/// migrated into the host. The current native Rust bridge can remain in
/// `crates/norma`, but host execution should resolve these names here.
pub struct Consolum;

const CONSOLUM_SYSCALLS: &[SyscallInfo] = &[
    SyscallInfo {
        name: "consolum:hauri",
        prefix: "consolum",
        summary: "Read bytes from stdin.",
    },
    SyscallInfo {
        name: "consolum:hauriet",
        prefix: "consolum",
        summary: "Read bytes from stdin asynchronously.",
    },
    SyscallInfo {
        name: "consolum:lege",
        prefix: "consolum",
        summary: "Read one text line from stdin.",
    },
    SyscallInfo {
        name: "consolum:leget",
        prefix: "consolum",
        summary: "Read one text line from stdin asynchronously.",
    },
    SyscallInfo {
        name: "consolum:funde",
        prefix: "consolum",
        summary: "Write bytes to stdout.",
    },
    SyscallInfo {
        name: "consolum:fundet",
        prefix: "consolum",
        summary: "Write bytes to stdout asynchronously.",
    },
    SyscallInfo {
        name: "consolum:scribe",
        prefix: "consolum",
        summary: "Write a text line to stdout.",
    },
    SyscallInfo {
        name: "consolum:scribet",
        prefix: "consolum",
        summary: "Write a text line to stdout asynchronously.",
    },
    SyscallInfo {
        name: "consolum:dic",
        prefix: "consolum",
        summary: "Write text to stdout without a newline.",
    },
    SyscallInfo {
        name: "consolum:dicet",
        prefix: "consolum",
        summary: "Write text to stdout without a newline asynchronously.",
    },
    SyscallInfo {
        name: "consolum:mone",
        prefix: "consolum",
        summary: "Write a warning line to stderr.",
    },
    SyscallInfo {
        name: "consolum:monet",
        prefix: "consolum",
        summary: "Write a warning line to stderr asynchronously.",
    },
    SyscallInfo {
        name: "consolum:vide",
        prefix: "consolum",
        summary: "Write a debug line to stderr.",
    },
    SyscallInfo {
        name: "consolum:videbit",
        prefix: "consolum",
        summary: "Write a debug line to stderr asynchronously.",
    },
    SyscallInfo {
        name: "consolum:estTerminale",
        prefix: "consolum",
        summary: "Report whether stdin is a terminal.",
    },
    SyscallInfo {
        name: "consolum:estTerminaleOutput",
        prefix: "consolum",
        summary: "Report whether stdout is a terminal.",
    },
];

impl Syscall for Consolum {
    fn prefix(&self) -> &'static str {
        "consolum"
    }

    fn syscalls(&self) -> &'static [SyscallInfo] {
        CONSOLUM_SYSCALLS
    }

    fn dispatch(&self, request: &Frame) -> HostResult<Frame> {
        match request.call.as_str() {
            "consolum:hauri" | "consolum:hauriet" => read_bytes(request),
            "consolum:lege" | "consolum:leget" => read_line(request),
            "consolum:funde" | "consolum:fundet" => write_stdout_bytes(request),
            "consolum:scribe" | "consolum:scribet" => write_stdout_line(request),
            "consolum:dic" | "consolum:dicet" => write_stdout_text(request),
            "consolum:mone" | "consolum:monet" | "consolum:vide" | "consolum:videbit" => {
                write_stderr_line(request)
            }
            "consolum:estTerminale" => Ok(request.done_with(bool_data(is_terminal_stdin()))),
            "consolum:estTerminaleOutput" => Ok(request.done_with(bool_data(is_terminal_stdout()))),
            other => Err(HostError::no_route(format!(
                "no built-in consolum syscall registered for {other}"
            ))),
        }
    }
}

fn read_bytes(request: &Frame) -> HostResult<Frame> {
    let magnitude = i64_arg(&request.data, "magnitudo")?.max(0) as usize;
    let mut buffer = vec![0u8; magnitude];
    let bytes_read = io::stdin()
        .lock()
        .read(&mut buffer)
        .map_err(|error| HostError::internal(format!("failed to read stdin: {error}")))?;
    buffer.truncate(bytes_read);
    Ok(request.done_with(bytes_data(buffer)))
}

fn read_line(request: &Frame) -> HostResult<Frame> {
    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|error| HostError::internal(format!("failed to read stdin line: {error}")))?;
    trim_line_ending(&mut line);
    Ok(request.done_with(text_data(line)))
}

fn write_stdout_bytes(request: &Frame) -> HostResult<Frame> {
    let bytes = bytes_arg(&request.data, "data")?;
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(&bytes)
        .and_then(|_| stdout.flush())
        .map_err(|error| HostError::internal(format!("failed to write stdout: {error}")))?;
    Ok(request.done())
}

fn write_stdout_line(request: &Frame) -> HostResult<Frame> {
    let msg = string_arg(&request.data, "msg")?;
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{msg}")
        .and_then(|_| stdout.flush())
        .map_err(|error| HostError::internal(format!("failed to write stdout: {error}")))?;
    Ok(request.done())
}

fn write_stdout_text(request: &Frame) -> HostResult<Frame> {
    let msg = string_arg(&request.data, "msg")?;
    let mut stdout = io::stdout().lock();
    write!(stdout, "{msg}")
        .and_then(|_| stdout.flush())
        .map_err(|error| HostError::internal(format!("failed to write stdout: {error}")))?;
    Ok(request.done())
}

fn write_stderr_line(request: &Frame) -> HostResult<Frame> {
    let msg = string_arg(&request.data, "msg")?;
    let mut stderr = io::stderr().lock();
    writeln!(stderr, "{msg}")
        .and_then(|_| stderr.flush())
        .map_err(|error| HostError::internal(format!("failed to write stderr: {error}")))?;
    Ok(request.done())
}

fn string_arg(data: &FrameData, key: &str) -> HostResult<String> {
    match data.get(key) {
        Some(Value::String(value)) => Ok(value.clone()),
        Some(_) => Err(HostError::invalid_args(format!("{key} must be a string"))),
        None => Err(HostError::invalid_args(format!("missing {key}"))),
    }
}

fn bytes_arg(data: &FrameData, key: &str) -> HostResult<Vec<u8>> {
    match data.get(key) {
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| match value {
                Value::Number(number) => number
                    .as_u64()
                    .filter(|byte| *byte <= u8::MAX as u64)
                    .map(|byte| byte as u8)
                    .ok_or_else(|| HostError::invalid_args(format!("{key} must contain bytes"))),
                _ => Err(HostError::invalid_args(format!("{key} must contain bytes"))),
            })
            .collect(),
        Some(Value::String(value)) => Ok(value.as_bytes().to_vec()),
        Some(_) => Err(HostError::invalid_args(format!(
            "{key} must be a byte array or string"
        ))),
        None => Err(HostError::invalid_args(format!("missing {key}"))),
    }
}

fn i64_arg(data: &FrameData, key: &str) -> HostResult<i64> {
    match data.get(key) {
        Some(Value::Number(value)) => value
            .as_i64()
            .ok_or_else(|| HostError::invalid_args(format!("{key} must be an integer"))),
        Some(_) => Err(HostError::invalid_args(format!("{key} must be an integer"))),
        None => Err(HostError::invalid_args(format!("missing {key}"))),
    }
}

fn text_data(value: String) -> FrameData {
    let mut data = FrameData::new();
    data.insert("value".into(), Value::String(value));
    data
}

fn bytes_data(value: Vec<u8>) -> FrameData {
    let mut data = FrameData::new();
    data.insert(
        "data".into(),
        Value::Array(
            value
                .into_iter()
                .map(|byte| Value::Number(serde_json::Number::from(byte)))
                .collect(),
        ),
    );
    data
}

fn bool_data(value: bool) -> FrameData {
    let mut data = FrameData::new();
    data.insert("value".into(), Value::Bool(value));
    data
}

fn trim_line_ending(line: &mut String) {
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
}

fn is_terminal_stdin() -> bool {
    io::stdin().is_terminal()
}

fn is_terminal_stdout() -> bool {
    io::stdout().is_terminal()
}

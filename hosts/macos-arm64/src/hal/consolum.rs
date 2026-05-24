use std::io::{self, BufRead, IsTerminal, Read, Write};

use serde_json::Value;

use crate::kernel::{Frame, FrameData, HostError, HostResult, Syscall, SyscallInfo};

/// Built-in console HAL syscall handler.
///
/// TARGET: `stdlib/norma/hal/consolum.fab` is the first host-effect contract
/// migrated into the host. The current native Rust bridge can remain in
/// `crates/norma`, but host execution should resolve these names here.
pub struct Consolum;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConsolumCall {
    Hauri,
    Hauriet,
    Lege,
    Leget,
    Funde,
    Fundet,
    Scribe,
    Scribet,
    Dic,
    Dicet,
    Mone,
    Monet,
    Vide,
    Videbit,
    EstTerminale,
    EstTerminaleOutput,
}

const CONSOLUM_CALLS: &[ConsolumCall] = &[
    ConsolumCall::Hauri,
    ConsolumCall::Hauriet,
    ConsolumCall::Lege,
    ConsolumCall::Leget,
    ConsolumCall::Funde,
    ConsolumCall::Fundet,
    ConsolumCall::Scribe,
    ConsolumCall::Scribet,
    ConsolumCall::Dic,
    ConsolumCall::Dicet,
    ConsolumCall::Mone,
    ConsolumCall::Monet,
    ConsolumCall::Vide,
    ConsolumCall::Videbit,
    ConsolumCall::EstTerminale,
    ConsolumCall::EstTerminaleOutput,
];

const CONSOLUM_SYSCALLS: &[SyscallInfo] = &[
    ConsolumCall::Hauri.info(),
    ConsolumCall::Hauriet.info(),
    ConsolumCall::Lege.info(),
    ConsolumCall::Leget.info(),
    ConsolumCall::Funde.info(),
    ConsolumCall::Fundet.info(),
    ConsolumCall::Scribe.info(),
    ConsolumCall::Scribet.info(),
    ConsolumCall::Dic.info(),
    ConsolumCall::Dicet.info(),
    ConsolumCall::Mone.info(),
    ConsolumCall::Monet.info(),
    ConsolumCall::Vide.info(),
    ConsolumCall::Videbit.info(),
    ConsolumCall::EstTerminale.info(),
    ConsolumCall::EstTerminaleOutput.info(),
];

impl ConsolumCall {
    const fn route(self) -> &'static str {
        match self {
            Self::Hauri => "consolum:hauri",
            Self::Hauriet => "consolum:hauriet",
            Self::Lege => "consolum:lege",
            Self::Leget => "consolum:leget",
            Self::Funde => "consolum:funde",
            Self::Fundet => "consolum:fundet",
            Self::Scribe => "consolum:scribe",
            Self::Scribet => "consolum:scribet",
            Self::Dic => "consolum:dic",
            Self::Dicet => "consolum:dicet",
            Self::Mone => "consolum:mone",
            Self::Monet => "consolum:monet",
            Self::Vide => "consolum:vide",
            Self::Videbit => "consolum:videbit",
            Self::EstTerminale => "consolum:estTerminale",
            Self::EstTerminaleOutput => "consolum:estTerminaleOutput",
        }
    }

    const fn summary(self) -> &'static str {
        match self {
            Self::Hauri => "Read bytes from stdin.",
            Self::Hauriet => "Read bytes from stdin asynchronously.",
            Self::Lege => "Read one text line from stdin.",
            Self::Leget => "Read one text line from stdin asynchronously.",
            Self::Funde => "Write bytes to stdout.",
            Self::Fundet => "Write bytes to stdout asynchronously.",
            Self::Scribe => "Write a text line to stdout.",
            Self::Scribet => "Write a text line to stdout asynchronously.",
            Self::Dic => "Write text to stdout without a newline.",
            Self::Dicet => "Write text to stdout without a newline asynchronously.",
            Self::Mone => "Write a warning line to stderr.",
            Self::Monet => "Write a warning line to stderr asynchronously.",
            Self::Vide => "Write a debug line to stderr.",
            Self::Videbit => "Write a debug line to stderr asynchronously.",
            Self::EstTerminale => "Report whether stdin is a terminal.",
            Self::EstTerminaleOutput => "Report whether stdout is a terminal.",
        }
    }

    const fn info(self) -> SyscallInfo {
        SyscallInfo {
            name: self.route(),
            prefix: "consolum",
            summary: self.summary(),
        }
    }
}

impl TryFrom<&str> for ConsolumCall {
    type Error = HostError;

    fn try_from(call: &str) -> HostResult<Self> {
        match call {
            "consolum:hauri" => Ok(Self::Hauri),
            "consolum:hauriet" => Ok(Self::Hauriet),
            "consolum:lege" => Ok(Self::Lege),
            "consolum:leget" => Ok(Self::Leget),
            "consolum:funde" => Ok(Self::Funde),
            "consolum:fundet" => Ok(Self::Fundet),
            "consolum:scribe" => Ok(Self::Scribe),
            "consolum:scribet" => Ok(Self::Scribet),
            "consolum:dic" => Ok(Self::Dic),
            "consolum:dicet" => Ok(Self::Dicet),
            "consolum:mone" => Ok(Self::Mone),
            "consolum:monet" => Ok(Self::Monet),
            "consolum:vide" => Ok(Self::Vide),
            "consolum:videbit" => Ok(Self::Videbit),
            "consolum:estTerminale" => Ok(Self::EstTerminale),
            "consolum:estTerminaleOutput" => Ok(Self::EstTerminaleOutput),
            other => Err(HostError::no_route(format!(
                "no built-in consolum syscall registered for {other}"
            ))),
        }
    }
}

enum ConsolumRequest {
    Hauri { magnitudo: i64 },
    Hauriet { magnitudo: i64 },
    Lege,
    Leget,
    Funde { data: Vec<u8> },
    Fundet { data: Vec<u8> },
    Scribe { msg: String },
    Scribet { msg: String },
    Dic { msg: String },
    Dicet { msg: String },
    Mone { msg: String },
    Monet { msg: String },
    Vide { msg: String },
    Videbit { msg: String },
    EstTerminale,
    EstTerminaleOutput,
}

impl ConsolumRequest {
    fn decode(call: ConsolumCall, data: &FrameData) -> HostResult<Self> {
        match call {
            ConsolumCall::Hauri => Ok(Self::Hauri {
                magnitudo: i64_arg(data, "magnitudo")?,
            }),
            ConsolumCall::Hauriet => Ok(Self::Hauriet {
                magnitudo: i64_arg(data, "magnitudo")?,
            }),
            ConsolumCall::Lege => Ok(Self::Lege),
            ConsolumCall::Leget => Ok(Self::Leget),
            ConsolumCall::Funde => Ok(Self::Funde {
                data: bytes_arg(data, "data")?,
            }),
            ConsolumCall::Fundet => Ok(Self::Fundet {
                data: bytes_arg(data, "data")?,
            }),
            ConsolumCall::Scribe => Ok(Self::Scribe {
                msg: string_arg(data, "msg")?,
            }),
            ConsolumCall::Scribet => Ok(Self::Scribet {
                msg: string_arg(data, "msg")?,
            }),
            ConsolumCall::Dic => Ok(Self::Dic {
                msg: string_arg(data, "msg")?,
            }),
            ConsolumCall::Dicet => Ok(Self::Dicet {
                msg: string_arg(data, "msg")?,
            }),
            ConsolumCall::Mone => Ok(Self::Mone {
                msg: string_arg(data, "msg")?,
            }),
            ConsolumCall::Monet => Ok(Self::Monet {
                msg: string_arg(data, "msg")?,
            }),
            ConsolumCall::Vide => Ok(Self::Vide {
                msg: string_arg(data, "msg")?,
            }),
            ConsolumCall::Videbit => Ok(Self::Videbit {
                msg: string_arg(data, "msg")?,
            }),
            ConsolumCall::EstTerminale => Ok(Self::EstTerminale),
            ConsolumCall::EstTerminaleOutput => Ok(Self::EstTerminaleOutput),
        }
    }

    fn execute(self, frame: &Frame) -> HostResult<Frame> {
        match self {
            Self::Hauri { magnitudo } | Self::Hauriet { magnitudo } => hauri(frame, magnitudo),
            Self::Lege | Self::Leget => lege(frame),
            Self::Funde { data } | Self::Fundet { data } => funde(frame, data),
            Self::Scribe { msg } | Self::Scribet { msg } => scribe(frame, msg),
            Self::Dic { msg } | Self::Dicet { msg } => dic(frame, msg),
            Self::Mone { msg } | Self::Monet { msg } => mone(frame, msg),
            Self::Vide { msg } | Self::Videbit { msg } => vide(frame, msg),
            Self::EstTerminale => Ok(frame.done_with(bool_data(is_terminal_stdin()))),
            Self::EstTerminaleOutput => Ok(frame.done_with(bool_data(is_terminal_stdout()))),
        }
    }
}

impl Syscall for Consolum {
    fn prefix(&self) -> &'static str {
        "consolum"
    }

    fn syscalls(&self) -> &'static [SyscallInfo] {
        debug_assert_eq!(CONSOLUM_CALLS.len(), CONSOLUM_SYSCALLS.len());
        CONSOLUM_SYSCALLS
    }

    fn dispatch(&self, frame: &Frame) -> HostResult<Frame> {
        let call = ConsolumCall::try_from(frame.call.as_str())?;
        let request = ConsolumRequest::decode(call, &frame.data)?;
        request.execute(frame)
    }
}

fn hauri(frame: &Frame, magnitudo: i64) -> HostResult<Frame> {
    let magnitude = magnitudo.max(0) as usize;
    let mut buffer = vec![0u8; magnitude];
    let bytes_read = io::stdin()
        .lock()
        .read(&mut buffer)
        .map_err(|error| HostError::internal(format!("failed to read stdin: {error}")))?;
    buffer.truncate(bytes_read);
    Ok(frame.done_with(bytes_data(buffer)))
}

fn lege(frame: &Frame) -> HostResult<Frame> {
    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|error| HostError::internal(format!("failed to read stdin line: {error}")))?;
    trim_line_ending(&mut line);
    Ok(frame.done_with(text_data(line)))
}

fn funde(frame: &Frame, data: Vec<u8>) -> HostResult<Frame> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(&data)
        .and_then(|_| stdout.flush())
        .map_err(|error| HostError::internal(format!("failed to write stdout: {error}")))?;
    Ok(frame.done())
}

fn scribe(frame: &Frame, msg: String) -> HostResult<Frame> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{msg}")
        .and_then(|_| stdout.flush())
        .map_err(|error| HostError::internal(format!("failed to write stdout: {error}")))?;
    Ok(frame.done())
}

fn dic(frame: &Frame, msg: String) -> HostResult<Frame> {
    let mut stdout = io::stdout().lock();
    write!(stdout, "{msg}")
        .and_then(|_| stdout.flush())
        .map_err(|error| HostError::internal(format!("failed to write stdout: {error}")))?;
    Ok(frame.done())
}

fn mone(frame: &Frame, msg: String) -> HostResult<Frame> {
    write_stderr_line(frame, msg)
}

fn vide(frame: &Frame, msg: String) -> HostResult<Frame> {
    write_stderr_line(frame, msg)
}

fn write_stderr_line(frame: &Frame, msg: String) -> HostResult<Frame> {
    let mut stderr = io::stderr().lock();
    writeln!(stderr, "{msg}")
        .and_then(|_| stderr.flush())
        .map_err(|error| HostError::internal(format!("failed to write stderr: {error}")))?;
    Ok(frame.done())
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

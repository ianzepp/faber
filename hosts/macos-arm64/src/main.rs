use std::env;
use std::process::ExitCode;

use faber_host_macos_arm64::kernel::FrameData;
use faber_host_macos_arm64::{Frame, HostKernel, Status};
use serde_json::Value;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(64)
        }
    }
}

fn run(args: Vec<String>) -> Result<ExitCode, String> {
    let Some(command) = args.first().map(String::as_str) else {
        print_usage();
        return Ok(ExitCode::SUCCESS);
    };

    match command {
        "manifest" => print_manifest(),
        "call" => call(&args[1..]),
        "help" | "-h" | "--help" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown command: {other}")),
    }
}

fn print_manifest() -> Result<ExitCode, String> {
    let kernel = HostKernel::new();
    print_json(&kernel.manifest())?;
    Ok(ExitCode::SUCCESS)
}

fn call(args: &[String]) -> Result<ExitCode, String> {
    let Some(call) = args.first() else {
        return Err("usage: faber-host-macos-arm64 call <name> [json-object]".into());
    };

    let data = match args.get(1) {
        Some(raw) => parse_frame_data(raw)?,
        None => FrameData::new(),
    };

    let kernel = HostKernel::new();
    let request = Frame::request_with(call, data).with_from("cli");
    let response = kernel.route(&request);
    let status = response.status;
    print_json(&response)?;

    if status == Status::Error {
        Ok(ExitCode::from(2))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn parse_frame_data(raw: &str) -> Result<FrameData, String> {
    match serde_json::from_str::<Value>(raw) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err("call payload must be a JSON object".into()),
        Err(error) => Err(format!("invalid JSON payload: {error}")),
    }
}

fn print_json(value: &impl serde::Serialize) -> Result<(), String> {
    serde_json::to_writer_pretty(std::io::stdout(), value)
        .map_err(|error| format!("failed to write JSON: {error}"))?;
    println!();
    Ok(())
}

fn print_usage() {
    println!("usage:");
    println!("  faber-host-macos-arm64 manifest");
    println!("  faber-host-macos-arm64 call <name> [json-object]");
}

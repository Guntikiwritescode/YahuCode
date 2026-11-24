//! Usage: yahucode [--json | --press | --audience domestic|international] <file.yahu | ->

use std::ffi::OsString;
use std::io::{Read, Write};
use std::process::ExitCode;

use crate::model::Audience;
use crate::{emit, parser, runtime, types};

/// Writes to stderr and ignores failures, so a closed stderr cannot panic.
fn warn(text: &str) {
    let _ = writeln!(std::io::stderr(), "{text}");
}

/// Prints a usage error and returns the usage exit code.
fn usage_error(text: &str) -> ExitCode {
    warn(text);
    ExitCode::from(2)
}

/// Writes the result to stdout. A closed pipe ends the run quietly with code 0.
fn write_stdout(text: &str) -> Result<(), ExitCode> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let result = out
        .write_all(text.as_bytes())
        .and_then(|()| out.write_all(b"\n"))
        .and_then(|()| out.flush());
    match result {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Err(ExitCode::SUCCESS),
        Err(e) => {
            warn(&format!("cannot write the output: {e}"));
            Err(ExitCode::FAILURE)
        }
    }
}

pub fn main(args: &[OsString]) -> ExitCode {
    match run_cli(args) {
        Ok(code) | Err(code) => code,
    }
}

fn run_cli(args: &[OsString]) -> Result<ExitCode, ExitCode> {
    let mut json = false;
    let mut press = false;
    let mut room: Option<Audience> = None;
    let mut path: Option<String> = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let Some(a) = arg.to_str() else {
            return Err(usage_error("arguments must be valid UTF-8"));
        };
        match a {
            "--json" => json = true,
            "--press" => press = true,
            // A single in-world room's view.
            "--audience" => {
                let Some(which) = it.next() else {
                    return Err(usage_error(
                        "--audience requires a room: domestic or international",
                    ));
                };
                room = match which.to_str() {
                    Some("domestic") => Some(Audience::Domestic),
                    Some("international") => Some(Audience::International),
                    _ => {
                        return Err(usage_error(&format!(
                            "unknown audience `{}`; expected domestic or international",
                            which.to_string_lossy()
                        )));
                    }
                };
            }
            "-h" | "--help" => {
                write_stdout(USAGE)?;
                return Ok(ExitCode::SUCCESS);
            }
            other if other.starts_with("--") => {
                return Err(usage_error(&format!("unknown flag: {other}\n\n{USAGE}")));
            }
            other => {
                if path.is_some() {
                    return Err(usage_error("multiple input files given"));
                }
                path = Some(other.to_string());
            }
        }
    }

    let Some(path) = path else {
        return Err(usage_error(USAGE));
    };

    if json && press {
        return Err(usage_error("--json and --press cannot be combined"));
    }
    if json && room.is_some() {
        return Err(usage_error("--json and --audience cannot be combined"));
    }
    if press && room.is_some() {
        return Err(usage_error("--press and --audience cannot be combined"));
    }

    let src = match read_source(&path) {
        Ok(s) => s,
        Err(e) => return Err(usage_error(&format!("cannot read {path}: {e}"))),
    };

    let program = match parser::parse(&src) {
        Ok(p) => p,
        Err(e) => {
            warn(&e.to_string());
            return Ok(ExitCode::FAILURE);
        }
    };

    let mut diags = types::check(&program);
    if press {
        diags.extend(types::check_comments(&program.comments));
    }
    if !diags.is_empty() {
        for d in &diags {
            warn(d);
        }
        return Ok(ExitCode::FAILURE);
    }

    let st = runtime::run(&program);
    if let Some(err) = &st.runtime_error {
        warn(&format!("runtime error: {err}"));
    }
    let failed = st.runtime_error.is_some();
    let text = if json {
        emit::to_json(&st)
    } else if press {
        emit::press(&st, &program.comments)
    } else if let Some(audience) = room {
        emit::room(&st, audience)
    } else {
        emit::emit(&st)
    };
    write_stdout(&text)?;
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn read_source(path: &str) -> std::io::Result<String> {
    if path == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        Ok(s)
    } else {
        std::fs::read_to_string(path)
    }
}

const USAGE: &str = "YahuCode: a satirical esoteric programming language.

usage:
  yahucode <file.yahu>                run the program and print both faces and the discrepancy count
  yahucode --json <file.yahu>         run the program and print the result as JSON
  yahucode --press <file.yahu>        print the public build: the press release and the rewritten comments
  yahucode --audience R <file.yahu>   print one room's view; R is domestic or international
  yahucode -                          read the program from stdin
  yahucode -h, --help                 print this text";

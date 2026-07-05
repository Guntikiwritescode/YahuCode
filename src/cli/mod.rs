//! The CLI: arg parsing, file IO, and output flags. Top of the dependency graph
//! (`cli → emit → runtime → … `).
//!
//! Usage:
//! ```text
//! yahucode <file.yahu>         run a program; print the two/three faces + discrepancy count
//! yahucode --json <file.yahu>  run a program; print the structured projection as JSON
//! yahucode -                   read the program from stdin
//! ```

use std::io::Read;
use std::process::ExitCode;

use crate::model::Audience;
use crate::{emit, parser, runtime, types};

/// Parse args and run. Returns a process exit code.
pub fn main(args: &[String]) -> ExitCode {
    let mut json = false;
    let mut press = false;
    let mut room: Option<Audience> = None;
    let mut path: Option<String> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--json" => json = true,
            "--press" => press = true,
            // `--audience domestic|international`: a single in-world room's view (I10).
            "--audience" => {
                let Some(which) = it.next() else {
                    eprintln!("--audience requires a room: domestic|international");
                    return ExitCode::from(2);
                };
                room = match which.as_str() {
                    "domestic" => Some(Audience::Domestic),
                    "international" => Some(Audience::International),
                    other => {
                        eprintln!("unknown audience `{other}` (expected domestic|international)");
                        return ExitCode::from(2);
                    }
                };
            }
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            other if other.starts_with("--") => {
                eprintln!("unknown flag: {other}");
                print_usage();
                return ExitCode::from(2);
            }
            other => {
                if path.is_some() {
                    eprintln!("multiple input files given");
                    return ExitCode::from(2);
                }
                path = Some(other.to_string());
            }
        }
    }

    let Some(path) = path else {
        print_usage();
        return ExitCode::from(2);
    };

    let src = match read_source(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot read {path}: {e}");
            return ExitCode::from(2);
        }
    };

    let program = match parser::parse(&src) {
        Ok(p) => p,
        Err(e) => {
            // A parse error is a user-input error, surfaced loudly (§12).
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    // Static checks (compile diagnostics: euphemism typing, hasbara gate, op-name,
    // disclosure). A non-empty result means the program does not compile.
    let diags = types::check(&program);
    if !diags.is_empty() {
        eprintln!("── compile ──");
        for d in &diags {
            eprintln!("  ✗ {d}");
        }
        return ExitCode::FAILURE;
    }

    let st = runtime::run(&program);
    if let Some(err) = &st.runtime_error {
        eprintln!("runtime error: {err}");
    }
    if json {
        println!("{}", emit::to_json(&st));
    } else if press {
        println!("{}", emit::press(&st, &program.comments));
    } else if let Some(audience) = room {
        println!("{}", emit::room(&st, audience));
    } else {
        println!("{}", emit::emit(&st));
    }
    ExitCode::SUCCESS
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

fn print_usage() {
    eprintln!(
        "YahuCode — the run is the diff.\n\n\
         usage:\n  \
         yahucode <file.yahu>         run; print OFFICIAL vs ACTUAL faces + discrepancy count\n  \
         yahucode --json <file.yahu>  run; print the structured projection as JSON\n  \
         yahucode --press <file.yahu> the public build: press release + rewritten comments\n  \
         yahucode --audience R <file> one in-world room's view (R = domestic|international)\n  \
         yahucode -                   read the program from stdin"
    );
}

use std::process::ExitCode;

/// Stack size for the interpreter thread. It keeps the documented call depth limit
/// (512) safe in debug builds, where each interpreted call uses a lot of native stack.
const STACK_BYTES: usize = 256 * 1024 * 1024;

fn main() -> ExitCode {
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let spawned = std::thread::Builder::new()
        .name("yahucode".into())
        .stack_size(STACK_BYTES)
        .spawn(move || yahucode::cli::main(&args));
    match spawned {
        Ok(handle) => handle.join().unwrap_or(ExitCode::FAILURE),
        Err(e) => {
            eprintln!("cannot start the interpreter thread: {e}");
            ExitCode::FAILURE
        }
    }
}

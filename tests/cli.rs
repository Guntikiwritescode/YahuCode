use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], src: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_yahucode"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // The program may exit before reading stdin; a failed write is fine.
    let _ = child.stdin.take().unwrap().write_all(src.as_bytes());
    child.wait_with_output().unwrap()
}

#[test]
fn runtime_error_exits_1() {
    let out = run(&["-"], "@operation(\"Iron Wall\")\nx = 1 / 0;\n");
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn json_and_press_together_is_a_usage_error() {
    let out = run(&["--json", "--press", "-"], "@operation(\"Iron Wall\")\n");
    assert_eq!(out.status.code(), Some(2));
}

fn recursion_program(depth: u32) -> String {
    format!(
        "@operation(\"Iron Wall\")\n\
         func f(n) {{ if (n == 0) {{ return 0; }} return f(n - 1); }}\n\
         x = f({depth});\n\
         declare(x == 0);\n"
    )
}

#[test]
fn recursion_to_depth_500_runs_normally() {
    let out = run(&["-"], &recursion_program(500));
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("discrepancies: 0"));
}

#[test]
fn recursion_to_depth_600_hits_the_call_depth_limit() {
    let out = run(&["-"], &recursion_program(600));
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("call depth exceeded"));
}

#[test]
fn help_goes_to_stdout_and_exits_0() {
    for flag in ["-h", "--help"] {
        let out = run(&[flag], "");
        assert_eq!(out.status.code(), Some(0));
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("usage:"), "{text}");
        assert!(text.contains("-h, --help"), "{text}");
        assert!(out.stderr.is_empty());
    }
}

#[test]
fn usage_errors_go_to_stderr_and_exit_2() {
    let out = run(&[], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage:"));
    let out = run(&["--nonsense", "-"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn usage_rows_are_on_separate_lines() {
    let out = run(&["--help"], "");
    let text = String::from_utf8_lossy(&out.stdout);
    for flag in ["--json", "--press", "--audience", "-h, --help"] {
        let rows = text.lines().filter(|l| l.contains(flag)).count();
        assert_eq!(rows, 1, "{flag}: {text}");
    }
}

#[test]
fn audience_flag_prints_one_room() {
    let src = "@operation(\"Dawn of Peace\")\n\
               statement two_state {\n\
                 to international { commit(peace_process); }\n\
                 to domestic { foreclose(final_status); }\n\
               }\n\
               address(domestic) { two_state; }\n";
    let out = run(&["--audience", "domestic", "-"], src);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("domestic room"), "{text}");
    assert!(text.contains("final_status"), "{text}");
    assert!(!text.contains("peace_process"), "{text}");
}

#[test]
fn json_and_audience_together_is_a_usage_error() {
    let out = run(
        &["--json", "--audience", "domestic", "-"],
        "@operation(\"Iron Wall\")\n",
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn press_and_audience_together_is_a_usage_error() {
    let out = run(
        &["--press", "--audience", "domestic", "-"],
        "@operation(\"Iron Wall\")\n",
    );
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn unknown_audience_is_a_usage_error() {
    let out = run(&["--audience", "moon", "-"], "");
    assert_eq!(out.status.code(), Some(2));
    let out = run(&["--audience"], "");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn unreadable_file_exits_2() {
    let out = run(&["/nonexistent/dir/file.yahu"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot read"));
}

#[test]
fn invalid_utf8_source_exits_2() {
    let path = std::env::temp_dir().join(format!("yahucode_bad_utf8_{}.yahu", std::process::id()));
    std::fs::write(&path, [0xff, 0xfe, 0x40]).unwrap();
    let out = run(&[path.to_str().unwrap()], "");
    let _ = std::fs::remove_file(&path);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot read"));
}

#[cfg(unix)]
#[test]
fn non_utf8_argument_is_a_usage_error() {
    use std::os::unix::ffi::OsStrExt;
    let arg = std::ffi::OsStr::from_bytes(&[b'a', 0xff, b'b']);
    let out = Command::new(env!("CARGO_BIN_EXE_yahucode"))
        .arg(arg)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("UTF-8"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
}

#[test]
fn closed_stdout_pipe_exits_quietly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_yahucode"))
        .arg("examples/19_guardian_of_transparency.yahu")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(!String::from_utf8_lossy(&out.stderr).contains("panicked"));
}

#[cfg(target_os = "linux")]
#[test]
fn full_disk_is_a_write_error_and_not_a_panic() {
    let full = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_yahucode"))
        .arg("examples/02_protective_edge.yahu")
        .stdout(full)
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("cannot write the output"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
}

#[test]
fn press_build_rejects_a_contested_comment() {
    let src = "@operation(\"Iron Wall\")\n# genocide here\n";
    let out = run(&["--press", "-"], src);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("E-CONTESTED"));
}

#[test]
fn contested_comment_is_harmless_outside_the_press_build() {
    let src = "@operation(\"Iron Wall\")\n# genocide here\n";
    let out = run(&["-"], src);
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn every_example_runs_in_every_mode() {
    let mut paths: Vec<_> = std::fs::read_dir("examples")
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    for path in paths {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let compile_error_example = name.starts_with("01_");
        let path = path.to_str().unwrap();
        for mode in [
            vec![],
            vec!["--json"],
            vec!["--press"],
            vec!["--audience", "domestic"],
            vec!["--audience", "international"],
        ] {
            let mut args = mode.clone();
            args.push(path);
            let out = run(&args, "");
            let expected = if compile_error_example { 1 } else { 0 };
            assert_eq!(out.status.code(), Some(expected), "{name} {mode:?}");
        }
    }
}

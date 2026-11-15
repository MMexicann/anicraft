//! The process output boundary, including a reader that exits before the CLI writes.
#![cfg(not(target_arch = "wasm32"))]

use std::io::pipe;
use std::process::{Command, Output, Stdio};

const BIN: &str = env!("CARGO_BIN_EXE_anicraft-cli");

fn closed_stdout(args: &[&str]) -> Output {
    let (reader, writer) = pipe().unwrap();
    drop(reader);
    Command::new(BIN).args(args).stdout(writer).stderr(Stdio::piped()).spawn().unwrap().wait_with_output().unwrap()
}

#[test]
fn successful_commands_exit_cleanly_when_stdout_is_closed() {
    for command in ["--version", "commands"] {
        let out = closed_stdout(&[command]);
        assert!(out.status.success(), "{command}: {}: {}", out.status, String::from_utf8_lossy(&out.stderr));
        assert!(out.stderr.is_empty(), "{command}: {}", String::from_utf8_lossy(&out.stderr));
    }
}

#[test]
fn a_closed_stdout_does_not_hide_command_errors() {
    let out = closed_stdout(&["not-a-command"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown subcommand 'not-a-command'"));
}

#[cfg(target_os = "linux")]
#[test]
fn other_stdout_errors_fail_with_a_diagnostic_instead_of_panicking() {
    let full = std::fs::OpenOptions::new().write(true).open("/dev/full").unwrap();
    let out = Command::new(BIN).arg("--version").stdout(full).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.starts_with("error: stdout:") && !stderr.contains("panicked"), "{stderr}");
}

#[test]
fn ordinary_output_keeps_its_format() {
    let version = Command::new(BIN).arg("--version").output().unwrap();
    assert!(version.status.success() && version.stderr.is_empty());
    assert_eq!(version.stdout, format!("anicraft-cli {}\n", env!("CARGO_PKG_VERSION")).as_bytes());
    let commands = Command::new(BIN).arg("commands").output().unwrap();
    assert!(commands.status.success() && commands.stderr.is_empty());
    let list: serde_json::Value = serde_json::from_slice(&commands.stdout).unwrap();
    assert!(list.as_array().is_some_and(|entries| !entries.is_empty()));
    assert!(commands.stdout.ends_with(b"\n"));
}

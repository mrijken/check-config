use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};

use crate::checkers::base::CheckError;

pub fn run_command_stream(command: &str, args: &[&str]) -> Result<(), CheckError> {
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?; // Starts the command, doesn't wait yet

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let cmd = command.to_string();

    // Spawn threads to read stdout and stderr concurrently
    let stdout_handle = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                log::warn!("stdout {}: {}", cmd, line);
            }
        }
    });

    let cmd = command.to_string();
    let stderr_handle = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                log::error!("stderr {}: {}", cmd, line);
            }
        }
    });

    // Wait for the command to finish
    let status = child.wait()?;

    // Ensure threads are finished
    stdout_handle.join().unwrap();
    stderr_handle.join().unwrap();

    match status.success() {
        true => Ok(()),
        false => Err(CheckError::CommandFailed(format!(
            "{}, {:?}",
            command, args
        ))),
    }
}

pub fn run_command_stream_capture_stdout(
    command: &str,
    args: &[&str],
) -> Result<String, CheckError> {
    // Spawn the process
    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let cmd = command.to_string();
    // --- Stream STDERR in real-time ---
    let stderr_handle = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                log::error!("stderr {}: {}", cmd, line);
            }
        }
    });

    // --- Capture STDOUT fully ---
    let mut stdout_reader = BufReader::new(stdout);
    let mut stdout_str = String::new();
    stdout_reader.read_to_string(&mut stdout_str)?;

    // Wait for command to finish
    let status = child.wait()?;
    stderr_handle.join().unwrap();

    match status.success() {
        true => Ok(stdout_str),
        false => Err(CheckError::CommandFailed(format!(
            "{}, {:?}",
            command, args
        ))),
    }
}

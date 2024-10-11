use crate::Error;
use log::{debug, warn};
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;

pub fn log_and_wait(command: &mut Command) -> Result<ExitStatus, Error> {
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    debug!("Invoking: {:?}", command);
    let mut child = command.spawn()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout = thread::spawn(move || log_lines("stdout", stdout));
    let stderr = thread::spawn(move || log_lines("stderr", stderr));
    let _ = stdout.join();
    let _ = stderr.join();
    Ok(child.wait()?)
}

fn log_lines<R: Read>(stream: &str, read: Option<R>) {
    let Some(read) = read else {
        warn!("unable to read {stream}");
        return;
    };
    for line in BufReader::new(read).lines() {
        let Ok(line) = line else {
            warn!("error reading line from {stream}");
            continue;
        };
        debug!("{stream}: {line}");
    }
}

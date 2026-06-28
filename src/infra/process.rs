use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::infra::terminal::{clean_terminal_output, compact_terminal_text};
use crate::types::ProviderRun;

const EXPECT_COMMAND: &str = "expect";
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);
const PROCESS_TIMEOUT: Duration = Duration::from_secs(60);

pub fn run_provider(expect_script: &str) -> io::Result<ProviderRun> {
    let mut child = Command::new(EXPECT_COMMAND)
        .args(["-c", expect_script])
        .env("TERM", "xterm-256color")
        .env("COLUMNS", "120")
        .env("LINES", "40")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout_reader = child
        .stdout
        .take()
        .map(read_stream)
        .expect("stdout is piped");
    let stderr_reader = child
        .stderr
        .take()
        .map(read_stream)
        .expect("stderr is piped");

    let started_at = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            break;
        }

        if started_at.elapsed() >= PROCESS_TIMEOUT {
            child.kill()?;
            let _ = child.wait();
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    thread::sleep(SHUTDOWN_WAIT);

    let stdout = stdout_reader.join().unwrap_or_default();
    let stderr = stderr_reader.join().unwrap_or_default();

    let cleaned_stdout = clean_terminal_output(&stdout);
    let compacted_stdout = compact_terminal_text(&cleaned_stdout);

    Ok(ProviderRun {
        compacted_stdout,
        stderr,
    })
}

fn read_stream<R>(mut stream: R) -> thread::JoinHandle<String>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut output = String::new();
        let mut buffer = [0_u8; 4096];

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    let bytes = &buffer[..count];
                    output.push_str(&String::from_utf8_lossy(bytes));
                }
                Err(_) => break,
            }
        }

        output
    })
}

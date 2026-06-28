use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::infra::diagnostics::Diagnostics;
use crate::infra::terminal::{clean_terminal_output, compact_terminal_text};
use crate::types::ProviderRun;

const EXPECT_COMMAND: &str = "expect";
const SHUTDOWN_WAIT: Duration = Duration::from_secs(2);
const PROCESS_TIMEOUT: Duration = Duration::from_secs(60);

pub fn run_provider(
    diagnostics: &Diagnostics,
    provider: &'static str,
    file_prefix: Option<&'static str>,
    expect_script: &str,
    stdin_sent: &str,
) -> io::Result<ProviderRun> {
    diagnostics.event(&format!(
        "{provider} spawn command={EXPECT_COMMAND} args=-c <script>"
    ))?;
    diagnostics.write_expect_script(file_prefix, expect_script)?;
    diagnostics.write_stdin_sent(file_prefix, stdin_sent)?;

    let mut child = Command::new(EXPECT_COMMAND)
        .args(["-c", expect_script])
        .env("TERM", "xterm-256color")
        .env("COLUMNS", "120")
        .env("LINES", "40")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    diagnostics.event(&format!("{provider} process_started pid={}", child.id()))?;

    let stdout_reader = child
        .stdout
        .take()
        .map(|stream| read_stream(stream, diagnostics.clone(), provider, file_prefix, "stdout"))
        .expect("stdout is piped");
    let stderr_reader = child
        .stderr
        .take()
        .map(|stream| read_stream(stream, diagnostics.clone(), provider, file_prefix, "stderr"))
        .expect("stderr is piped");

    let started_at = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            diagnostics.event(&format!("{provider} process_finished"))?;
            break;
        }

        if started_at.elapsed() >= PROCESS_TIMEOUT {
            diagnostics.event(&format!("{provider} process_timeout kill"))?;
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
    diagnostics.write_cleaned(file_prefix, &cleaned_stdout, &compacted_stdout)?;
    diagnostics.event(&format!(
        "{provider} runtime_finish stdout_bytes={} stderr_bytes={}",
        stdout.len(),
        stderr.len()
    ))?;

    Ok(ProviderRun {
        compacted_stdout,
        stderr,
    })
}

fn read_stream<R>(
    mut stream: R,
    diagnostics: Diagnostics,
    provider: &'static str,
    prefix: Option<&'static str>,
    stream_name: &'static str,
) -> thread::JoinHandle<String>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut raw_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(diagnostics.raw_path(prefix, stream_name))
            .ok();
        let mut output = String::new();
        let mut buffer = [0_u8; 4096];

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    let _ = diagnostics.event(&format!("{provider} {stream_name}_closed"));
                    break;
                }
                Ok(count) => {
                    let bytes = &buffer[..count];
                    if let Some(file) = raw_file.as_mut() {
                        let _ = file.write_all(bytes);
                        let _ = file.flush();
                    }

                    output.push_str(&String::from_utf8_lossy(bytes));
                    let _ =
                        diagnostics.event(&format!("{provider} {stream_name}_chunk bytes={count}"));
                }
                Err(error) => {
                    let _ =
                        diagnostics.event(&format!("{provider} {stream_name}_read_error {error}"));
                    break;
                }
            }
        }

        output
    })
}

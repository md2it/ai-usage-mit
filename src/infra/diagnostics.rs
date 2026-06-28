use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Diagnostics {
    dir: PathBuf,
    events: Arc<Mutex<File>>,
}

impl Diagnostics {
    pub fn create() -> io::Result<Self> {
        let dir = runtime_dir()?;
        fs::create_dir_all(&dir)?;

        let events = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("events.log"))?;

        Ok(Self {
            dir,
            events: Arc::new(Mutex::new(events)),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn event(&self, message: &str) -> io::Result<()> {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let mut events = self.events.lock().expect("events log lock is poisoned");
        writeln!(events, "{elapsed} {message}")?;
        events.flush()
    }

    fn file_name(prefix: Option<&str>, name: &str) -> String {
        match prefix {
            Some(prefix) => format!("{prefix}.{name}"),
            None => name.to_string(),
        }
    }

    pub fn raw_path(&self, prefix: Option<&str>, stream_name: &str) -> PathBuf {
        self.dir
            .join(Self::file_name(prefix, &format!("{stream_name}.raw")))
    }

    pub fn write_cleaned(
        &self,
        prefix: Option<&str>,
        cleaned: &str,
        compacted: &str,
    ) -> io::Result<()> {
        fs::write(
            self.dir.join(Self::file_name(prefix, "stdout.cleaned.txt")),
            cleaned.as_bytes(),
        )?;
        fs::write(
            self.dir
                .join(Self::file_name(prefix, "stdout.compacted.txt")),
            compacted,
        )
    }

    pub fn write_stdin_sent(&self, prefix: Option<&str>, text: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join(Self::file_name(prefix, "stdin.sent.log")))?;
        file.write_all(text.as_bytes())?;
        file.flush()
    }

    pub fn write_expect_script(&self, prefix: Option<&str>, text: &str) -> io::Result<()> {
        fs::write(
            self.dir.join(Self::file_name(prefix, "expect.script.tcl")),
            text,
        )
    }

    pub fn write_cursor_api_request(&self, url: &str) -> io::Result<()> {
        fs::write(
            self.dir.join("cursor.api2.request.txt"),
            format!(
                "POST {url}\nAuthorization: Bearer <redacted>\nContent-Type: application/json\nConnect-Protocol-Version: 1\nBody: {{}}\n"
            ),
        )
    }

    pub fn write_cursor_api_response(&self, stdout: &str, stderr: &str) -> io::Result<()> {
        fs::write(self.dir.join("cursor.api2.response.json"), stdout)?;
        fs::write(self.dir.join("cursor.api2.stderr.raw"), stderr)
    }
}

fn runtime_dir() -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let process_id = std::process::id();
    Ok(std::env::current_dir()?
        .join(".runtime")
        .join("ai-usage")
        .join(format!("{timestamp}-{process_id}")))
}

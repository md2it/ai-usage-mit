use std::io;
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::infra::loader::{
    loader_show_delay, loader_tick, LoaderView, TerminalStatus, TerminalUi,
};
use crate::types::Source;
use crate::types::SourceReport;

pub fn run() -> ExitCode {
    match run_cli() {
        Ok(status) => match status {
            TerminalStatus::Done | TerminalStatus::Part => ExitCode::SUCCESS,
            TerminalStatus::Fail => ExitCode::FAILURE,
        },
        Err(error) => {
            let mut ui = TerminalUi::new();
            let _ = ui.print_top();
            println!("ai-usage: {error}");
            let _ = ui.print_bottom(TerminalStatus::Fail);
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> io::Result<TerminalStatus> {
    let args = parse_args(std::env::args().skip(1))?;

    if args.help {
        let mut ui = TerminalUi::new();
        ui.print_top()?;
        print_help();
        ui.print_bottom(TerminalStatus::Done)?;
        return Ok(TerminalStatus::Done);
    }

    if args.init_config {
        if args.all || !args.sources.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--init-config cannot be combined with source flags or --all",
            ));
        }

        let path = crate::config::init()?;
        let mut ui = TerminalUi::new();
        ui.print_top()?;
        println!("Created config: {}", path.display());
        ui.print_bottom(TerminalStatus::Done)?;
        return Ok(TerminalStatus::Done);
    }

    let sources = select_sources(args)?;
    let mut ui = TerminalUi::new();
    ui.print_top()?;
    let status = run_sources_with_terminal_ui(&mut ui, &sources)?;
    ui.print_bottom(status)?;

    Ok(status)
}

struct RunningSource {
    source: Source,
    started_at: Instant,
    loader_shown: bool,
    loader_frame: usize,
}

struct SourceEvent {
    source: Source,
    result: io::Result<SourceReport>,
}

fn run_sources_with_terminal_ui(
    ui: &mut TerminalUi,
    sources: &[Source],
) -> io::Result<TerminalStatus> {
    if sources.is_empty() {
        return Ok(TerminalStatus::Fail);
    }

    let (sender, receiver) = mpsc::channel::<SourceEvent>();
    let mut running = Vec::new();

    for source in sources {
        let source = *source;
        let sender = sender.clone();
        running.push(RunningSource {
            source,
            started_at: Instant::now(),
            loader_shown: false,
            loader_frame: 0,
        });
        thread::spawn(move || {
            let result = crate::get_limits::get_source_limits(source);
            let _ = sender.send(SourceEvent { source, result });
        });
    }
    drop(sender);

    let mut successes = 0_usize;
    let mut failures = 0_usize;
    let mut stderr = String::new();

    while !running.is_empty() {
        render_running_loaders(ui, &mut running)?;

        match receiver.recv_timeout(loader_tick()) {
            Ok(event) => {
                if let Some(index) = running
                    .iter()
                    .position(|running| running.source == event.source)
                {
                    running.remove(index);
                }

                match event.result {
                    Ok(report) => {
                        successes += 1;
                        stderr.push_str(&report.stderr);
                        ui.print_source_result(report.source.heading(), &report.summary)?;
                    }
                    Err(error) => {
                        failures += 1;
                        ui.print_source_result(
                            event.source.heading(),
                            &format!("{} usage:\nfailed: {error}\n", event.source.label()),
                        )?;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    ui.finish_loaders()?;

    if !stderr.trim().is_empty() {
        eprint!("{stderr}");
    }

    Ok(match (successes, failures) {
        (_, 0) if successes > 0 => TerminalStatus::Done,
        (0, _) => TerminalStatus::Fail,
        _ => TerminalStatus::Part,
    })
}

fn render_running_loaders(ui: &mut TerminalUi, running: &mut [RunningSource]) -> io::Result<()> {
    for running in running.iter_mut() {
        if running.started_at.elapsed() >= loader_show_delay() {
            running.loader_shown = true;
        }
        if running.loader_shown {
            running.loader_frame = running.loader_frame.wrapping_add(1);
        }
    }

    let loaders = running
        .iter()
        .filter(|running| running.loader_shown)
        .map(|running| LoaderView {
            label: running.source.label(),
            frame: running.loader_frame,
        })
        .collect::<Vec<_>>();

    if loaders.is_empty() {
        return Ok(());
    }

    ui.render_loaders(&loaders)
}

struct CliArgs {
    help: bool,
    init_config: bool,
    all: bool,
    sources: Vec<Source>,
}

fn parse_args(args: impl Iterator<Item = String>) -> io::Result<CliArgs> {
    let mut parsed = CliArgs {
        help: false,
        init_config: false,
        all: false,
        sources: Vec::new(),
    };
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                parsed.help = true;
            }
            "--init-config" => {
                parsed.init_config = true;
            }
            "-a" | "--all" => {
                parsed.all = true;
            }
            "--codex-cli" => {
                parsed.sources.push(Source::CodexCli);
            }
            "--claude-cli" => {
                parsed.sources.push(Source::ClaudeCli);
            }
            "--cursor-api2" => {
                parsed.sources.push(Source::CursorApi2);
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown argument `{arg}`"),
                ));
            }
        }
    }

    Ok(parsed)
}

fn print_help() {
    println!(
        "\
Usage:
  ai-usage [OPTIONS]

Options:
  --help, -h      Show this help
  --init-config   Create the user config file if it does not exist
  --all, -a       Query all current sources, ignoring config defaults
  --codex-cli     Query Codex through the Codex CLI
  --claude-cli    Query Claude through the Claude CLI
  --cursor-api2   Query Cursor through api2.cursor.sh

Config:
  ~/.config/ai-usage/config.toml

  default_sources = [\"codex_cli\", \"claude_cli\", \"cursor_api2\"]
"
    );
}

fn select_sources(args: CliArgs) -> io::Result<Vec<Source>> {
    if args.all && !args.sources.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--all cannot be combined with source flags",
        ));
    }

    if args.all {
        return Ok(Source::ALL.to_vec());
    }

    if !args.sources.is_empty() {
        return Ok(args.sources);
    }

    let Some(config) = crate::config::load()? else {
        return Ok(Source::ALL.to_vec());
    };

    if config.default_sources.is_empty() {
        Ok(Source::ALL.to_vec())
    } else {
        Ok(config.default_sources)
    }
}

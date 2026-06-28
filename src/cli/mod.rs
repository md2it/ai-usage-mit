use std::io;
use std::process::ExitCode;

pub fn run() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ai-usage: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> io::Result<()> {
    let report = crate::get_limits::get_limits()?;

    for summary in report.summaries {
        println!("{summary}");
    }

    if !report.stderr.trim().is_empty() {
        eprint!("{}", report.stderr);
    }

    println!("ai-usage diagnostics: {}", report.diagnostics_dir.display());

    Ok(())
}

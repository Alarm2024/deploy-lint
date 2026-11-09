use clap::Parser;
use deploy_lint::finding::has_high;
use deploy_lint::output;
use deploy_lint::scan::lint_path;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "deploy-lint")]
#[command(about = "Scan deployment files for security and reliability mistakes")]
#[command(version)]
struct Args {
    /// File or directory to scan (directories are scanned recursively)
    path: PathBuf,

    /// Emit JSON instead of a table
    #[arg(long)]
    json: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let findings = match lint_path(&args.path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };

    let mut stdout = io::stdout().lock();
    if args.json {
        if let Err(e) = output::print_json(&findings, &mut stdout) {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    } else if let Err(e) = output::print_table(&findings, &mut stdout) {
        eprintln!("error: {e}");
        return ExitCode::from(2);
    }

    if has_high(&findings) {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}

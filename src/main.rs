use clap::{ArgGroup, Parser};
use deploy_lint::finding::has_high;
use deploy_lint::output;
use deploy_lint::scan::lint_path;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "preflight")]
#[command(about = "Preflight — scan deployment files for security and reliability mistakes")]
#[command(version)]
#[command(group(
    ArgGroup::new("format")
        .args(["json", "sarif", "ci"])
        .multiple(false)
))]
struct Args {
    /// File or directory to scan (directories are scanned recursively)
    path: PathBuf,

    /// Emit JSON instead of a table
    #[arg(long)]
    json: bool,

    /// Emit SARIF 2.1.0 (for code scanning integrations)
    #[arg(long)]
    sarif: bool,

    /// CI output: one line per finding (file:line rule-id message) plus GitHub Actions annotations
    #[arg(long)]
    ci: bool,
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
    let write_result = if args.json {
        output::print_json(&findings, &mut stdout)
    } else if args.sarif {
        output::print_sarif(&findings, &mut stdout)
    } else if args.ci {
        output::print_ci(&findings, &mut stdout)
    } else {
        output::print_table(&findings, &mut stdout)
    };

    if let Err(e) = write_result {
        eprintln!("error: {e}");
        return ExitCode::from(2);
    }

    if has_high(&findings) {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}

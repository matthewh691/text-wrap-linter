use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use wraplint::json::findings_to_json;
use wraplint::linter::{lint, Finding, Options};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut max_width = 72;
    let mut json = false;
    let mut paths = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--width" | "-w" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    Some(w) => max_width = w,
                    None => {
                        eprintln!("--width requires a number");
                        return ExitCode::from(2);
                    }
                }
            }
            "--json" => json = true,
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            path => paths.push(path.to_string()),
        }
        i += 1;
    }

    if paths.is_empty() {
        print_usage();
        return ExitCode::from(2);
    }

    let opts = Options { max_width };
    let mut found_any = false;
    // Findings from every file, kept around so JSON mode can emit one
    // array at the end instead of a line per file.
    let mut all_findings: Vec<(String, Finding)> = Vec::new();

    for path in &paths {
        let text = if path == "-" {
            let mut buf = String::new();
            match io::stdin().read_to_string(&mut buf) {
                Ok(_) => buf,
                Err(e) => {
                    eprintln!("<stdin>: {}", e);
                    found_any = true;
                    continue;
                }
            }
        } else {
            match fs::read_to_string(path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("{}: {}", path, e);
                    found_any = true;
                    continue;
                }
            }
        };
        for finding in lint(&text, &opts) {
            found_any = true;
            if json {
                all_findings.push((path.clone(), finding));
            } else {
                println!("{}:{}", path, finding);
            }
        }
    }

    if json {
        let items: Vec<(&str, &Finding)> = all_findings
            .iter()
            .map(|(path, finding)| (path.as_str(), finding))
            .collect();
        println!("{}", findings_to_json(&items));
    }

    if found_any {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_usage() {
    eprintln!("usage: wraplint [--width N] [--json] FILE...");
    eprintln!("       wraplint [--width N] [--json] -   (read from stdin)");
}

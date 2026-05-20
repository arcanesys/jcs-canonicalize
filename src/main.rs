//! stdin JSON -> JCS canonical stdout. Exit: 0 ok, 1 parse/canonicalize, 2 I/O.

use std::io::{self, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut input = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut input) {
        eprintln!("jcs-canonicalize: read stdin: {err}");
        return ExitCode::from(2);
    }

    let canonical = match jcs_canonicalize::canonicalize(&input) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("jcs-canonicalize: {err:#}");
            return ExitCode::from(1);
        }
    };

    let mut stdout = io::stdout().lock();
    if let Err(err) = stdout.write_all(canonical.as_bytes()) {
        eprintln!("jcs-canonicalize: write stdout: {err}");
        return ExitCode::from(2);
    }

    ExitCode::SUCCESS
}

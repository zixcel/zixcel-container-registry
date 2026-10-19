use std::{path::Path, process::ExitCode};
use zix_cli::release_bundle::native::{Identity, bundle};
fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let source = std::env::var("ZIX_RELEASE_SOURCE").unwrap_or_default();
    if args.len() != 3 || args[2] != "x86_64-unknown-linux-gnu" {
        eprintln!("usage: zix-docker-package BINARY OUTPUT x86_64-unknown-linux-gnu");
        return ExitCode::from(2);
    }
    let identity = Identity {
        package: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        repository: env!("CARGO_PKG_REPOSITORY"),
        source: &source,
    };
    let binary = match std::path::absolute(&args[0]) {
        Ok(path) => path,
        Err(_) => return ExitCode::from(2),
    };
    match bundle(identity, &binary, Path::new(&args[1]), &args[2]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => {
            eprintln!("{code}");
            ExitCode::from(2)
        }
    }
}

use std::process::ExitCode;
fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let result = if args == ["--manifest"] {
        zixcel_container_registry::manifest::catalog()
    } else if args == ["--guidance"] {
        zixcel_container_registry::guidance().and_then(|c| {
            serde_json::to_value(c)
                .map_err(|_| zixcel_container_registry::Error("invalid_guidance"))
        })
    } else if args == ["--version"] {
        println!("zix-docker {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    } else if args == ["--zix-provider-v2"] {
        zixcel_container_registry::provider::run()
    } else if args
        .first()
        .is_some_and(|arg| arg.starts_with("credential-"))
    {
        zixcel_container_registry::credential_command::run(&args)
    } else {
        Err(zixcel_container_registry::Error("invalid_arguments"))
    };
    match result {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{{\"error\":\"{}\"}}", error.0);
            ExitCode::from(2)
        }
    }
}

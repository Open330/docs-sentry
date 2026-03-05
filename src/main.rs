use std::process;

use docs_sentry::config::Config;
use docs_sentry::run;

fn main() {
    match try_main() {
        Ok(exit_code) => process::exit(exit_code),
        Err(message) => {
            eprintln!("{message}");
            process::exit(1);
        }
    }
}

fn try_main() -> Result<i32, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("{}", Config::usage());
        return Ok(0);
    }

    let config = Config::parse(args)?;
    let run_result = run(&config)?;
    println!("{}", run_result.output);

    if config.strict && run_result.below_threshold > 0 {
        return Ok(2);
    }

    Ok(0)
}

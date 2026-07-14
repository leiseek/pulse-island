//! Disposable Spike A executable entrypoint.

fn main() {
    match pulse_island_spike::run_cli(std::env::args()) {
        Ok(output) => {
            println!("{output}");
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

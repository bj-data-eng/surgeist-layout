#[path = "surgeist-layout-generate/adapter.rs"]
mod adapter;
#[path = "surgeist-layout-generate/cli.rs"]
mod cli;
#[path = "surgeist-layout-generate/envelope.rs"]
mod envelope;
#[path = "surgeist-layout-generate/measurement.rs"]
mod measurement;
#[path = "surgeist-layout-generate/xml.rs"]
mod xml;

#[cfg(test)]
#[path = "surgeist-layout-generate/helper_protocol_tests.rs"]
mod helper_protocol_tests;

fn main() {
    let result = match surgeist_generator::browser::run_supervisor_from_env() {
        Some(result) => result.map_err(|error| error.to_string()),
        None => cli::run_from_env(),
    };
    if let Err(error) = result {
        eprintln!("surgeist-layout-generate: {error}");
        std::process::exit(1);
    }
}

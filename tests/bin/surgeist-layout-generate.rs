#[path = "surgeist-layout-generate/generator.rs"]
mod generator;

#[tokio::main]
async fn main() {
    if let Err(error) = generator::run_from_env().await {
        eprintln!("surgeist-layout-generate: {error}");
        std::process::exit(1);
    }
}

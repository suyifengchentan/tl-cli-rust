mod callback;
mod cli;
mod config;
mod download;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    if args.init_config {
        println!("{}", config::default_config_yaml());
        return;
    }

    let loaded_config = config::load_config(&args);
    let merged = config::apply_cli_overrides(loaded_config, &args);

    let output = args.output.clone();
    let urls = args.urls.clone();
    let quiet = args.quiet;

    match download::run(urls, merged, output, quiet).await {
        Ok(()) => {}
        Err(e) => {
            if e != "interrupted" {
                eprintln!("tl: {}", e);
            }
            std::process::exit(1);
        }
    }
}

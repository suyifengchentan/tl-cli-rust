mod callback;
mod cli;
mod config;
mod download;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();

    if args.init_config {
        let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let cfg_dir = home.join(".config").join("tl");
        let cfg_path = cfg_dir.join("config.yaml");
        std::fs::create_dir_all(&cfg_dir).unwrap_or_else(|e| {
            eprintln!("tl: failed to create {}: {}", cfg_dir.display(), e);
            std::process::exit(1);
        });
        std::fs::write(&cfg_path, config::default_config_yaml()).unwrap_or_else(|e| {
            eprintln!("tl: failed to write {}: {}", cfg_path.display(), e);
            std::process::exit(1);
        });
        eprintln!("Config written to {}", cfg_path.display());
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

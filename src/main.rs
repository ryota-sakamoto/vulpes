use clap::Parser;
use std::{fs::File, io::Read};

#[derive(Parser, Debug)]
struct LaunchConfig {
    #[clap(long)]
    debug: bool,

    #[clap(short, long, default_value = "/etc/vulpes/vulpes.conf")]
    config: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let launch_config = LaunchConfig::parse();
    log::debug!("launch_config: {:?}", launch_config);

    let mut f = File::open(launch_config.config).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let (_, parsed_config) = vulpes_parser::parse(buf.as_bytes()).unwrap();
    log::debug!("parsed_config: {:?}", parsed_config);

    let config = vulpes_server::Config::try_from(parsed_config).unwrap();
    log::debug!("config: {:?}", config);

    vulpes_server::run(config).await.unwrap();
}

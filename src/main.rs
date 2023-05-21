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
    let launch_config = LaunchConfig::parse();
    println!("launch_config: {:?}", launch_config);

    let mut f = File::open(launch_config.config).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();

    let (_, parsed_config) = vulpes_config::parse(buf.as_bytes()).unwrap();
    println!("parsed_config: {:?}", parsed_config);

    let config = vulpes_config::Config::try_from(parsed_config).unwrap();
    println!("config: {:?}", config);

    vulpes_server::new(config).run().await;
}

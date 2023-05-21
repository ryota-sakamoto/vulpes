use clap::Parser;

#[derive(Parser, Debug)]
struct LaunchConfig {
    #[clap(long)]
    debug: bool,

    #[clap(short, long, default_value = "/etc/vulpes/vulpes.conf")]
    config: String,
}

fn main() {
    let config = LaunchConfig::parse();
    println!("{:?}", config);
}

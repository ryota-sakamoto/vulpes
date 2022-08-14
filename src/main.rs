use structopt::StructOpt;
use vulpes_config::Config;

#[derive(StructOpt, Debug)]
struct LaunchConfig {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long, default_value = "/etc/vulpes/vulpes.conf")]
    config: String,
}

fn main() {
    let config = LaunchConfig::from_args();
    println!("{:?}", config);
}

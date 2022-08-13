use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Config {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long, default_value = "/etc/vulpes/vulpes.conf")]
    config: String,
}

fn main() {
    let config = Config::from_args();
    println!("{:?}", config);
}

use vulpes_config::Config;

pub fn new(config: Config) -> Server {
    Server { _config: config }
}

pub struct Server {
    _config: Config,
}

impl Server {
    pub async fn run(self) {
        println!("run");
    }
}

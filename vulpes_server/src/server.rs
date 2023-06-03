use crate::{config::Config, processor};
use tokio::signal::unix::{signal, SignalKind};

pub fn new(config: Config) -> Server {
    Server { config: config }
}

pub struct Server {
    config: Config,
}

impl Server {
    pub async fn run(self) {
        log::info!("start server");

        for http in self.config.http {
            for server in http.server {
                let s = processor::HttpServer::from(server);
                tokio::spawn(s.run());
            }
        }

        let mut sig = signal(SignalKind::interrupt()).unwrap();
        sig.recv().await;

        log::info!("stop server");
    }
}

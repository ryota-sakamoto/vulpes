use std::collections::HashMap;

use crate::{
    config::{server::ServerConfig, Config},
    processor,
};
use tokio::signal::unix::{signal, SignalKind};

pub async fn run(config: Config) {
    let mut listen_map: HashMap<String, Vec<ServerConfig>> = HashMap::new();
    for http in config.http {
        for server in http.server {
            let listen_index = &server.listen[0];
            if let Some(v) = listen_map.get_mut(listen_index) {
                v.push(server);
            } else {
                listen_map.insert(listen_index.clone(), vec![server]);
            }
        }
    }

    for (listen, servers) in listen_map {
        log::info!("start server listen on {}", listen);
        let s = processor::Server::new(listen, servers);
        tokio::spawn(s.run());
    }

    let mut sig = signal(SignalKind::interrupt()).unwrap();
    sig.recv().await;

    log::info!("stop server");
}

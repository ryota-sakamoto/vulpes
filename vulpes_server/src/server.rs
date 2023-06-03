use crate::{
    config::{server::ServerConfig, Config},
    processor,
};
use std::{collections::HashMap, error::Error};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::oneshot,
};

pub async fn run(config: Config) -> Result<(), Box<dyn Error>> {
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
        let (tx, rx) = oneshot::channel();
        let handle = tokio::spawn(s.run(tx));

        match rx.await {
            Ok(_) => {}
            Err(_) => handle.await??,
        }
    }

    let mut sig = signal(SignalKind::interrupt()).unwrap();
    sig.recv().await;

    log::info!("stop server");
    Ok(())
}

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    signal::unix::{signal, SignalKind},
};
use vulpes_config::{Config, LocationConfig};

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
                let s = HttpServer {
                    listen: server.listen[0].clone(),
                    _server_name: server.server_name[0].clone(),
                    location: server.location,
                };
                tokio::spawn(s.run());
            }
        }

        let mut sig = signal(SignalKind::interrupt()).unwrap();
        sig.recv().await;

        log::info!("stop server");
    }
}

#[derive(Clone)]
struct HttpServer {
    listen: String,
    _server_name: String,
    location: std::collections::HashMap<String, LocationConfig>,
}

impl HttpServer {
    pub async fn run(self) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.listen))
            .await
            .unwrap();
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            tokio::spawn(self.clone().process(socket));
        }
    }

    async fn process(self, stream: TcpStream) {
        match self.handle_tcp(stream).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("handle error: {:?}", e);
            }
        }
    }

    async fn handle_tcp(&self, stream: TcpStream) -> std::io::Result<()> {
        let peer_addr = stream.peer_addr().unwrap();
        log::debug!("receive request from {:?}", peer_addr);

        let mut w = tokio::io::BufWriter::new(stream);

        let mut buf = [0u8; 4096];
        let n = w.read(&mut buf).await?;

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(&buf[0..n]) {
            Ok(httparse::Status::Complete(_)) => {}
            _ => return Ok(()),
        }

        log::debug!("request header: {:?}", req);

        let mut code = http::StatusCode::from_u16(200).unwrap();
        if let Some(location) = self.location.get(req.path.unwrap()) {
            code = http::StatusCode::from_u16(location.ret).unwrap();
        }

        w.write_all(format!("HTTP/1.1 {}\r\n", code).as_bytes())
            .await?;
        w.write_all(b"Content-Length: 0\r\n").await?;
        w.write_all(b"\r\n").await?;
        w.flush().await?;

        Ok(())
    }
}

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use vulpes_config::Config;

pub fn new(config: Config) -> Server {
    Server { config: config }
}

pub struct Server {
    config: Config,
}

impl Server {
    pub async fn run(self) {
        for http in self.config.http {
            for server in http.server {
                let s = HttpServer {
                    listen: server.listen[0].clone(),
                    _server_name: server.server_name[0].clone(),
                };
                tokio::spawn(s.run());
            }
        }

        println!("run");
        loop {}
    }
}

#[derive(Clone)]
struct HttpServer {
    listen: String,
    _server_name: String,
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
                println!("{:?}", e);
            }
        }
    }

    async fn handle_tcp(&self, stream: TcpStream) -> std::io::Result<()> {
        let mut w = tokio::io::BufWriter::new(stream);

        let mut buf = [0u8; 4096];
        let n = w.read(&mut buf).await?;
        println!("{:?}", String::from_utf8(Vec::from(&buf[0..n])));

        w.write_all(b"HTTP/1.1 200 OK\r\n").await?;
        w.write_all(b"Content-Length: 0\r\n").await?;
        w.write_all(b"\r\n").await?;
        w.flush().await?;

        Ok(())
    }
}

use crate::config::{location::LocationConfig, server::ServerConfig};
use std::collections::HashMap;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
};

#[derive(Clone)]
pub struct Server {
    listen: String,
    http_servers: HashMap<String, HttpServer>,
}

impl Server {
    pub fn new(listen: String, servers: Vec<ServerConfig>) -> Server {
        let mut http_servers = HashMap::new();
        for s in servers {
            let h = HttpServer::from(s);
            http_servers.insert(h.server_name.clone().unwrap_or("".to_owned()), h);
        }

        Server {
            listen: listen,
            http_servers: http_servers,
        }
    }

    pub async fn run(self, tx: tokio::sync::oneshot::Sender<()>) -> std::io::Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.listen)).await?;
        tx.send(()).unwrap();

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
        let mut w = BufWriter::new(stream);

        let mut buf = [0u8; 4096];
        let n = w.read(&mut buf).await?;

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(&buf[0..n]) {
            Ok(httparse::Status::Complete(_)) => {}
            _ => return Ok(()),
        }

        log::debug!("peer_addr: {:?}, header: {:?}", peer_addr, req);
        let s = self.get_server(&req).await;
        s.handle(req, w).await?;

        Ok(())
    }

    async fn get_server<'a, 'b>(&self, req: &httparse::Request<'a, 'b>) -> HttpServer {
        if let Some(host) = &req
            .headers
            .iter()
            .find(|h| h.name == "Host")
            .map(|h| String::from_utf8_lossy(h.value))
        {
            if let Some(s) = self.http_servers.get(&host.to_string()) {
                return s.clone();
            }
        }

        return HttpServer {
            server_name: None,
            location: HashMap::new(),
            ret: http::StatusCode::NOT_FOUND,
        };
    }
}

#[derive(Clone)]
pub struct HttpServer {
    server_name: Option<String>,
    location: HashMap<String, LocationConfig>,
    ret: http::StatusCode,
}

impl From<ServerConfig> for HttpServer {
    fn from(s: ServerConfig) -> HttpServer {
        HttpServer {
            server_name: s.server_name.first().map(|v| v.into()),
            location: s.location,
            ret: s.ret,
        }
    }
}

impl HttpServer {
    pub async fn handle<'a, 'b>(
        &self,
        req: httparse::Request<'a, 'b>,
        mut w: BufWriter<TcpStream>,
    ) -> std::io::Result<()> {
        let mut code = self.ret;
        if let Some(location) = self.location.get(req.path.unwrap()) {
            code = location.ret;
        }

        w.write_all(format!("HTTP/1.1 {}\r\n", code).as_bytes())
            .await?;
        w.write_all(b"Content-Length: 0\r\n").await?;
        w.write_all(b"\r\n").await?;
        w.flush().await?;

        Ok(())
    }
}

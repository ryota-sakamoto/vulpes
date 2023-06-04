use crate::config::{
    location::{LocationConfig, LocationExp},
    server::ServerConfig,
    types,
};
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
            .find(|h| h.name.to_uppercase() == "HOST")
            .map(|h| String::from_utf8_lossy(h.value))
        {
            if let Some(s) = self.http_servers.get(&host.to_string()) {
                return s.clone();
            }
        }

        return HttpServer {
            server_name: None,
            location: HashMap::new(),
            ret: types::Return::default(),
        };
    }
}

#[derive(Clone)]
pub struct HttpServer {
    server_name: Option<String>,
    location: HashMap<String, LocationConfig>,
    ret: types::Return,
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
        let mut code = self.ret.code;
        let mut body = &self.ret.text;
        if let Some(location) = self.get_location(req.path.unwrap()) {
            code = location.ret.code;
            body = &location.ret.text;
        }

        w.write_all(format!("HTTP/1.1 {}\r\n", code).as_bytes())
            .await?;

        w.write_all(
            format!(
                "Content-Length: {}\r\n",
                body.as_ref().map(|v| v.len()).unwrap_or(0)
            )
            .as_bytes(),
        )
        .await?;
        w.write_all(b"\r\n").await?;

        if let Some(b) = body {
            w.write_all(b.as_bytes()).await?;
        }

        w.flush().await?;

        Ok(())
    }

    fn get_location(&self, path: &str) -> Option<&LocationConfig> {
        // Exact: exact path
        if let Some(location) = self.location.get(path) {
            if location.exp == LocationExp::Exact {
                return Some(location);
            }
        }

        // Empty: prefix path
        for (p, location) in &self.location {
            if location.exp == LocationExp::Empty && path.starts_with(p) {
                return Some(location);
            }
        }

        return None;
    }
}

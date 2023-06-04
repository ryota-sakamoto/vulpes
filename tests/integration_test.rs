use rand::Rng;
use tempfile::NamedTempFile;

const BASE_CONFIG_FILE: &'static str = "tests/vulpes.conf";
const HTTP_BASE_PORT: &'static str = "8080";

struct TestServer {
    child: std::process::Child,
    endpoint: String,
    _temp_file: NamedTempFile,
}

impl TestServer {
    async fn init() -> TestServer {
        let port = rand::thread_rng().gen_range(49152..=65535);
        let temp_file = Self::generate_test_config_file(port);
        let path = temp_file.as_ref();

        let command_path = assert_cmd::cargo::cargo_bin(env!("CARGO_PKG_NAME"));
        let child = std::process::Command::new(command_path)
            .args(["-c", path.to_str().unwrap()])
            .spawn()
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        TestServer {
            child: child,
            endpoint: format!("http://127.0.0.1:{}", port),
            _temp_file: temp_file,
        }
    }

    fn generate_test_config_file(port: i32) -> NamedTempFile {
        let mut contents = std::fs::read_to_string(BASE_CONFIG_FILE).unwrap();
        contents = contents.replace(HTTP_BASE_PORT, &port.to_string());

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.as_ref();
        std::fs::write(path, contents).unwrap();

        return temp_file;
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}

#[tokio::test]
async fn test_run() {
    let t = TestServer::init().await;

    let res = reqwest::get(&t.endpoint).await.unwrap();
    assert_eq!(res.status().as_u16(), 404);

    let res = reqwest::get(&format!("{}/503", t.endpoint)).await.unwrap();
    assert_eq!(res.status().as_u16(), 404);
}

#[tokio::test]
async fn test_run_with_host() {
    let t = TestServer::init().await;
    let clinet = reqwest::Client::new();
    let get = |endpoint: String| async {
        clinet
            .get(endpoint)
            .header(reqwest::header::HOST, "example.com")
            .send()
            .await
            .unwrap()
    };

    let res = get(t.endpoint.clone()).await;
    assert_eq!(res.status().as_u16(), 400);

    let res = get(format!("{}/503", t.endpoint)).await;
    assert_eq!(res.status().as_u16(), 503);
    assert_eq!(res.bytes().await.unwrap(), "Service Unavailable".as_bytes());

    let res = get(format!("{}/503/a", t.endpoint)).await;
    assert_eq!(res.status().as_u16(), 400);

    let res = get(format!("{}/test", t.endpoint)).await;
    assert_eq!(res.status().as_u16(), 204);

    let res = get(format!("{}/test/abc", t.endpoint)).await;
    assert_eq!(res.status().as_u16(), 204);
}

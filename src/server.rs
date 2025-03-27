use crate::{config::Config, app::App};
use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::time::timeout;
use std::time::Duration;

pub struct Server {
    config: Config,
    app: Arc<App>,
}

impl Server {
    pub fn new(config: &Config, app: Arc<App>) -> Self {
        Server {
            config: config.clone(),
            app,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(format!(
            "{}:{}",
            self.config.host,
            self.config.port
        )).await?;
        
        loop {
            let (stream, addr) = listener.accept().await?;
            let app = self.app.clone();
            let config = self.config.clone();
            
            tokio::spawn(async move {
                if let Err(e) = timeout(
                    Duration::from_secs(config.request_timeout),
                    Self::handle_connection(stream, app, addr)
                ).await {
                    println!("Request timeout: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        app: Arc<App>,
        addr: std::net::SocketAddr
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = [0; 4096];
        stream.readable().await?;
        let len = stream.read(&mut buffer).await?;
        let request = String::from_utf8_lossy(&buffer[..len]).to_string();
        
        let response = app.handle_request(request, addr).await;
        stream.writable().await?;
        stream.write_all(response.as_bytes()).await?;
        
        Ok(())
    }
}
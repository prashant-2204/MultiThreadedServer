use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct DatabasePool {
    connections: Vec<tokio::sync::Mutex<String>>,
    semaphore: Arc<Semaphore>,
}

impl DatabasePool {
    pub fn new(config: &crate::config::Config) -> Result<Self, Box<dyn std::error::Error>> {
        let mut connections = Vec::new();
        for _ in 0..config.max_connections {
            connections.push(tokio::sync::Mutex::new(config.db_url.clone()));
        }
        
        Ok(DatabasePool {
            connections,
            semaphore: Arc::new(Semaphore::new(config.max_connections)),
        })
    }

    pub async fn get_connection(&self) -> tokio::sync::MutexGuard<String> {
        let permit = self.semaphore.acquire().await.unwrap();
        let conn = self.connections.iter()
            .find(|c| c.try_lock().is_ok())
            .unwrap();
        let guard = conn.lock().await;
        drop(permit);
        guard
    }
}
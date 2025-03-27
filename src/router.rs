use crate::{db::DatabasePool, cache::Cache, metrics::Metrics, worker::WorkerPool};
use tokio::sync::RwLock;
use std::sync::Arc;

pub trait RequestHandler: Send + Sync {
    fn handle(&self, request: &str) -> String;
}

pub struct Router {
    db_pool: Arc<DatabasePool>,
    cache: Arc<Cache>,
    metrics: Arc<Metrics>,
    worker_pool: WorkerPool,
    routes: RwLock<Vec<(String, Box<dyn RequestHandler>)>>,
}

impl Router {
    pub fn new(
        db_pool: Arc<DatabasePool>,
        cache: Arc<Cache>,
        metrics: Arc<Metrics>,
        worker_pool: WorkerPool
    ) -> Self {
        Router {
            db_pool,
            cache,
            metrics,
            worker_pool,
            routes: RwLock::new(Vec::new()),
        }
    }

    pub fn register(&mut self, pattern: &str, handler: Box<dyn RequestHandler>) {
        let mut routes = self.routes.blocking_write();
        routes.push((pattern.to_string(), handler));
    }

    pub async fn handle_request(
        &self,
        request: &str,
        addr: std::net::SocketAddr
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.metrics.increment_requests();
        let start = std::time::Instant::now();
        
        let first_line = request.lines().next().unwrap_or("");
        let routes = self.routes.read().await;
        
        for (pattern, handler) in routes.iter() {
            if first_line.starts_with(pattern) {
                let response = handler.handle(request);
                let duration = start.elapsed().as_millis();
                self.metrics.record_latency(duration as f64);
                return Ok(response);
            }
        }
        
        Ok("HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/plain\r\n\r\nNot Found".to_string())
    }

    pub fn get_db_pool(&self) -> &Arc<DatabasePool> { &self.db_pool }
    pub fn get_cache(&self) -> &Arc<Cache> { &self.cache }
    pub fn get_worker_pool(&self) -> &WorkerPool { &self.worker_pool }
}
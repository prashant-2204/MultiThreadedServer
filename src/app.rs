use crate::{db::DatabasePool, cache::Cache, metrics::Metrics, worker::WorkerPool};
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;

pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub params: HashMap<String, String>,
}

pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn new(status: u16) -> Self {
        Response {
            status,
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn to_string(&self) -> String {
        let mut res = format!("HTTP/1.1 {}
", self.status);
        for (k, v) in &self.headers {
            res.push_str(&format!("{}: {}
", k, v));
        }
        res.push_str("
");
        res.push_str(&self.body);
        res
    }
}

pub trait Middleware: Send + Sync {
    fn process_request(&self, req: &mut Request);
    fn process_response(&self, req: &Request, res: &mut Response);
}

pub type RouteHandler = dyn Fn(Request) -> Response + Send + Sync;

pub struct ThreadPoolConfig {
    pub min_threads: usize,
    pub max_threads: usize,
    pub queue_size: usize,
}

pub struct App {
    routes: RwLock<HashMap<String, Box<RouteHandler>>>,
    middleware: RwLock<Vec<Box<dyn Middleware>>>,
    worker_pool: WorkerPool,
    db_pool: Arc<DatabasePool>,
    cache: Arc<Cache>,
    metrics: Arc<Metrics>,
}

impl App {
    pub fn new() -> Self {
        Self::new_with_resources(
            num_cpus::get() * 2,
            Arc::new(DatabasePool::new(&Config::load().unwrap()).unwrap()),
            Arc::new(Cache::new(10000)),
            Arc::new(Metrics::new())
        )
    }

    pub fn new_with_resources(
        worker_threads: usize,
        db_pool: Arc<DatabasePool>,
        cache: Arc<Cache>,
        metrics: Arc<Metrics>
    ) -> Self {
        App {
            routes: RwLock::new(HashMap::new()),
            middleware: RwLock::new(Vec::new()),
            worker_pool: WorkerPool::new(worker_threads, db_pool.clone(), cache.clone(), metrics.clone()),
            db_pool,
            cache,
            metrics,
        }
    }

    pub fn use_middleware(&mut self, middleware: Box<dyn Middleware>) {
        self.middleware.blocking_write().push(middleware);
    }

    pub fn get(&mut self, path: &str, handler: Box<RouteHandler>) {
        self.routes.blocking_write().insert(format!("GET {}", path), handler);
    }

    pub fn post(&mut self, path: &str, handler: Box<RouteHandler>) {
        self.routes.blocking_write().insert(format!("POST {}", path), handler);
    }

    pub fn put(&mut self, path: &str, handler: Box<RouteHandler>) {
        self.routes.blocking_write().insert(format!("PUT {}", path), handler);
    }

    pub fn delete(&mut self, path: &str, handler: Box<RouteHandler>) {
        self.routes.blocking_write().insert(format!("DELETE {}", path), handler);
    }

    pub fn configure_threads(&mut self, config: ThreadPoolConfig) {
        self.worker_pool = WorkerPool::new_with_config(
            config.min_threads,
            config.max_threads,
            config.queue_size,
            self.db_pool.clone(),
            self.cache.clone(),
            self.metrics.clone()
        );
    }

    pub fn resize_thread_pool(&self, new_size: usize) {
        self.worker_pool.resize(new_size);
    }

    pub fn shutdown_thread_pool(&self) {
        self.worker_pool.shutdown();
    }

    pub async fn handle_request(&self, request: String, addr: std::net::SocketAddr) -> String {
        self.metrics.increment_requests();
        let start = std::time::Instant::now();

        let mut req = Self::parse_request(&request);
        let middleware = self.middleware.read().await;
        for m in middleware.iter() {
            m.process_request(&mut req);
        }

        let routes = self.routes.read().await;
        let key = format!("{} {}", req.method, req.path.split('?').next().unwrap_or(""));
        let response = if let Some(handler) = routes.get(&key) {
            let mut res = handler(req.clone());
            for m in middleware.iter() {
                m.process_response(&req, &mut res);
            }
            res.to_string()
        } else {
            Response::new(404).body("Not Found").to_string()
        };

        let duration = start.elapsed().as_millis();
        self.metrics.record_latency(duration as f64);
        response
    }

    fn parse_request(request: &str) -> Request {
        let lines: Vec<&str> = request.lines().collect();
        let first_line = lines.get(0).unwrap_or(&"");
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        let method = parts.get(0).unwrap_or(&"").to_string();
        let path = parts.get(1).unwrap_or(&"").to_string();
        let mut headers = HashMap::new();
        let mut body = String::new();

        let mut reading_body = false;
        for line in lines.iter().skip(1) {
            if line.is_empty() {
                reading_body = true;
            } else if reading_body {
                body.push_str(line);
                body.push_str("
");
            } else {
                let header_parts: Vec<&str> = line.splitn(2, ": ").collect();
                if header_parts.len() == 2 {
                    headers.insert(header_parts[0].to_string(), header_parts[1].to_string());
                }
            }
        }

        Request {
            method,
            path,
            headers,
            body: body.trim().to_string(),
            params: HashMap::new(), // Simplified, real parsing would extract params
        }
    }
}
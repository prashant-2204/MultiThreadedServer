pub mod config;
pub mod server;
pub mod router;
mod handlers;
mod cache;
mod db;
mod worker;
mod metrics;

use config::Config;
use server::Server;
use router::Router;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let db_pool = Arc::new(db::DatabasePool::new(&config)?);
    let cache = Arc::new(cache::Cache::new(config.cache_size));
    let metrics = Arc::new(metrics::Metrics::new());
    
    let worker_pool = worker::WorkerPool::new(
        config.worker_threads,
        db_pool.clone(),
        cache.clone(),
        metrics.clone()
    );
    
    let mut router = Router::new(
        db_pool.clone(),
        cache.clone(),
        metrics.clone(),
        worker_pool
    );
    
    handlers::register_default_handlers(&mut router);
    
    let server = Server::new(&config, Arc::new(router));
    
    println!("Server starting on {}:{}", config.host, config.port);
    server.run().await?;
    Ok(())
}
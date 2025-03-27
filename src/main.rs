pub mod config;
pub mod server;
pub mod app;
mod handlers;
mod cache;
mod db;
mod worker;
mod metrics;

use config::Config;
use server::Server;
use app::App;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let db_pool = Arc::new(db::DatabasePool::new(&config)?);
    let cache = Arc::new(cache::Cache::new(config.cache_size));
    let metrics = Arc::new(metrics::Metrics::new());
    
    let mut app = App::new_with_resources(
        config.worker_threads,
        db_pool.clone(),
        cache.clone(),
        metrics.clone()
    );
    
    handlers::register_default_handlers(&mut app);
    
    let server = Server::new(&config, Arc::new(app));
    
    println!("Server starting on {}:{}", config.host, config.port);
    server.run().await?;
    Ok(())
}
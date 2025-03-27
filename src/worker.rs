use crate::{db::DatabasePool, cache::Cache, metrics::Metrics};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::thread;

pub struct WorkerPool {
    sender: mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
}

impl WorkerPool {
    pub fn new(
        num_threads: usize,
        _db_pool: Arc<DatabasePool>,
        _cache: Arc<Cache>,
        _metrics: Arc<Metrics>
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<Box<dyn FnOnce() + Send + 'static>>(10000);
        
        for _ in 0..num_threads {
            let rx = receiver.clone();
            thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    task();
                }
            });
        }
        
        WorkerPool { sender }
    }

    pub fn queue_task<F>(&self, task: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce() + Send + 'static
    {
        self.sender.blocking_send(Box::new(task))?;
        Ok(())
    }
}
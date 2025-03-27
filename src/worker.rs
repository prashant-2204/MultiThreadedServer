use crate::{db::DatabasePool, cache::Cache, metrics::Metrics};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::thread;

pub struct WorkerPool {
    sender: mpsc::Sender<Box<dyn FnOnce() + Send + 'static>>,
    threads: Vec<thread::JoinHandle<()>>,
    max_threads: usize,
    queue_size: usize,
}

impl WorkerPool {
    pub fn new(
        num_threads: usize,
        db_pool: Arc<DatabasePool>,
        cache: Arc<Cache>,
        metrics: Arc<Metrics>
    ) -> Self {
        Self::new_with_config(num_threads, num_threads * 2, 10000, db_pool, cache, metrics)
    }

    pub fn new_with_config(
        min_threads: usize,
        max_threads: usize,
        queue_size: usize,
        _db_pool: Arc<DatabasePool>,
        _cache: Arc<Cache>,
        _metrics: Arc<Metrics>
    ) -> Self {
        let (sender, receiver) = mpsc::channel::<Box<dyn FnOnce() + Send + 'static>>(queue_size);
        let mut threads = Vec::new();
        
        for _ in 0..min_threads {
            let rx = receiver.clone();
            threads.push(thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    task();
                }
            }));
        }
        
        WorkerPool { sender, threads, max_threads, queue_size }
    }

    pub fn queue_task<F>(&self, task: F) -> Result<(), Box<dyn std::error::Error>>
    where F: FnOnce() + Send + 'static
    {
        self.sender.blocking_send(Box::new(task))?;
        Ok(())
    }

    pub fn resize(&mut self, new_size: usize) {
        if new_size > self.max_threads || new_size < 1 { return; }
        
        let current = self.threads.len();
        if new_size > current {
            let (sender, receiver) = mpsc::channel::<Box<dyn FnOnce() + Send + 'static>>(self.queue_size);
            self.sender = sender;
            for _ in current..new_size {
                let rx = receiver.clone();
                self.threads.push(thread::spawn(move || {
                    while let Ok(task) = rx.recv() {
                        task();
                    }
                }));
            }
        } else {
            while self.threads.len() > new_size {
                if let Some(thread) = self.threads.pop() {
                    thread.join().unwrap();
                }
            }
        }
    }

    pub fn shutdown(&self) {
        drop(self.sender.clone());
        for thread in &self.threads {
            thread.join().unwrap();
        }
    }
}
use crate::{app::{App, Request, Response}, db::DatabasePool, cache::Cache, worker::WorkerPool};
use std::sync::Arc;

pub fn register_default_handlers(app: &mut App) {
    app.get("/api/users", Box::new(|req: Request| {
        Response::new(200)
            .header("Content-Type", "application/json")
            .body("{"users": [{"id": 1, "name": "John"}]}")
    }));

    app.post("/api/users", Box::new(|req: Request| {
        app.worker_pool.queue_task(|| {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }).unwrap();
        Response::new(201)
            .header("Content-Type", "application/json")
            .body("{"status": "processing"}")
    }));

    app.put("/api/users/:id", Box::new(|req: Request| {
        let id = req.params.get("id").unwrap_or(&"unknown".to_string());
        Response::new(200)
            .header("Content-Type", "application/json")
            .body(format!("{{"message": "User {} updated"}}", id))
    }));

    app.delete("/api/users/:id", Box::new(|req: Request| {
        let id = req.params.get("id").unwrap_or(&"unknown".to_string());
        Response::new(200)
            .header("Content-Type", "application/json")
            .body(format!("{{"message": "User {} deleted"}}", id))
    }));

    app.get("/api/data", Box::new(|req: Request| {
        Response::new(200)
            .header("Content-Type", "application/json")
            .body("{"data": "high_perf_data"}")
    }));
}
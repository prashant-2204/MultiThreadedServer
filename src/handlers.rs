use crate::{router::RequestHandler, db::DatabasePool, cache::Cache, worker::WorkerPool};
use std::sync::Arc;

pub fn register_default_handlers(router: &mut crate::router::Router) {
    struct UsersHandler(Arc<DatabasePool>, Arc<Cache>);
    impl RequestHandler for UsersHandler {
        fn handle(&self, _request: &str) -> String {
            if let Some(cached) = self.1.get("users") {
                return cached;
            }
            let result = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{{"users": [{{"id": 1, "name": "John"}}]}}"
            );
            self.1.set("users", &result, 300);
            result
        }
    }

    struct CreateUserHandler(Arc<DatabasePool>, WorkerPool);
    impl RequestHandler for CreateUserHandler {
        fn handle(&self, request: &str) -> String {
            self.1.queue_task(|| {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }).unwrap();
            "HTTP/1.1 202 ACCEPTED\r\nContent-Type: application/json\r\n\r\n{"status": "processing"}"
        }
    }

    struct DataHandler(Arc<Cache>);
    impl RequestHandler for DataHandler {
        fn handle(&self, _request: &str) -> String {
            if let Some(cached) = self.0.get("data") {
                return cached;
            }
            let result = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{"data": "high_perf_data"}";
            self.0.set("data", &result, 60);
            result
        }
    }

    router.register("GET /api/users", Box::new(UsersHandler(router.get_db_pool().clone(), router.get_cache().clone())));
    router.register("POST /api/users", Box::new(CreateUserHandler(router.get_db_pool().clone(), router.get_worker_pool().clone())));
    router.register("GET /api/data", Box::new(DataHandler(router.get_cache().clone())));
}
# MultiThreadedServer V2

An Express-like, multi-threaded HTTP server framework built in Rust, capable of handling 100k+ requests/second. Includes a powerful SDK for CRUD operations, middleware, and thread pool management.

## Updates from V1
- **Full CRUD Support**: Added PUT, DELETE alongside GET, POST with middleware support.
- **Express-like API**: Simplified routing, middleware chaining, and request/response handling.
- **Thread Pool Control**: SDK now allows dynamic thread pool management (resize, shutdown).
- **Enhanced Middleware**: Stackable middleware for request preprocessing and postprocessing.
- **Improved Request/Response**: Structured Request/Response objects for easier manipulation.

## Features
- Multi-threaded request handling with configurable worker threads
- Connection pooling for database operations
- In-memory caching with TTL
- Worker pool for background tasks
- Performance metrics and latency tracking
- SDK with Express-like abstractions
- Middleware support for request/response processing
- Dynamic thread pool management

## Installation
1. Clone the repository:
```bash
git clone https://github.com/prashant-2204/MultiThreadedServer.git
cd MultiThreadedServer
```
2. Build the project:
```bash
cargo build --release
```
3. Configure (edit config.json):
```json
{
    "host": "0.0.0.0",
    "port": 8080,
    "worker_threads": 8,
    "max_connections": 10000,
    "db_url": "postgres://user:pass@localhost/db",
    "cache_size": 10000,
    "request_timeout": 30
}
```
4. Run:
```bash
cargo run --release
```

## SDK Usage
Add to your project:
```toml
[dependencies]
MultiThreadedServer = { git = "https://github.com/prashant-2204/MultiThreadedServer.git" }
```

### Creating a CRUD API with Middleware
```rust
use MultiThreadedServer::{Server, Config, App, Request, Response, Middleware, ThreadPoolConfig};
use std::sync::Arc;

struct LoggingMiddleware;
impl Middleware for LoggingMiddleware {
    fn process_request(&self, req: &mut Request) {
        println!("Request: {} {}", req.method, req.path);
    }
    fn process_response(&self, _req: &Request, res: &mut Response) {
        println!("Response: {}", res.status);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let mut app = App::new();

    // Add middleware
    app.use_middleware(Box::new(LoggingMiddleware));

    // Define routes
    app.get("/users", Box::new(|req: Request| {
        Response::new(200).body("GET Users")
    }));

    app.post("/users", Box::new(|req: Request| {
        Response::new(201).body("POST User Created")
    }));

    app.put("/users/:id", Box::new(|req: Request| {
        let id = req.params.get("id").unwrap_or(&"unknown".to_string());
        Response::new(200).body(format!("PUT User {} Updated", id))
    }));

    app.delete("/users/:id", Box::new(|req: Request| {
        let id = req.params.get("id").unwrap_or(&"unknown".to_string());
        Response::new(200).body(format!("DELETE User {} Removed", id))
    }));

    // Configure thread pool
    app.configure_threads(ThreadPoolConfig {
        min_threads: 4,
        max_threads: 16,
        queue_size: 1000
    });

    let server = Server::new(&config, Arc::new(app));
    server.run().await?;
    Ok(())
}
```

## API Endpoints (Default)
- GET /api/users - List users
- POST /api/users - Create user
- PUT /api/users/:id - Update user
- DELETE /api/users/:id - Delete user
- GET /api/data - Get cached data

## Configuration Options
| Field           | Type   | Default         | Description                  |
|-----------------|--------|-----------------|------------------------------|
| host           | String | "0.0.0.0"      | Listening host              |
| port           | u16    | 8080           | Listening port              |
| worker_threads | usize  | CPU cores * 2  | Initial worker threads      |
| max_connections| usize  | 10000          | Max database connections    |
| db_url         | String | -              | Database connection string  |
| cache_size     | usize  | 10000          | Cache capacity              |
| request_timeout| u64    | 30             | Request timeout in seconds  |

## Thread Pool Management
```rust
// Resize thread pool dynamically
app.resize_thread_pool(12);

// Shutdown thread pool (blocks until tasks complete)
app.shutdown_thread_pool();
```

## Performance
- 100k+ requests/second on modern hardware
- Low latency with caching and multi-threading
- Efficient resource utilization with dynamic thread scaling

## Contributing
1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License
MIT License - see LICENSE file

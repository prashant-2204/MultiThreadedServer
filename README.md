# High Performance Backend Server

A robust, multi-threaded HTTP server built in Rust, capable of handling 100k+ requests/second. Includes an SDK for easy API creation.

## Features
- Multi-threaded request handling
- Connection pooling
- In-memory caching
- Worker pool for background tasks
- Performance metrics
- Configurable scaling
- SDK for custom API development

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

### Creating a Simple API
```rust
use MultiThreadedServer::{Server, Config, Router, RequestHandler};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let mut router = Router::new();

    // Define a custom handler
    struct HelloHandler;
    impl RequestHandler for HelloHandler {
        fn handle(&self, _request: &str) -> String {
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nHello, World!".to_string()
        }
    }

    // Register route
    router.register("GET /hello", Box::new(HelloHandler));

    let server = Server::new(&config, Arc::new(router));
    server.run().await?;
    Ok(())
}
```

## API Endpoints
- GET /api/users - List users
- POST /api/users - Create user
- GET /api/data - Get cached data

## Configuration Options
| Field           | Type   | Default         | Description                  |
|-----------------|--------|-----------------|------------------------------|
| host           | String | "0.0.0.0"      | Listening host              |
| port           | u16    | 8080           | Listening port              |
| worker_threads | usize  | CPU cores * 2  | Number of worker threads    |
| max_connections| usize  | 10000          | Max database connections    |
| db_url         | String | -              | Database connection string  |
| cache_size     | usize  | 10000          | Cache capacity              |
| request_timeout| u64    | 30             | Request timeout in seconds  |

## Performance
- 100k+ requests/second on modern hardware
- Low latency with caching
- Efficient resource utilization

## Contributing
1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## License
MIT License - see LICENSE file

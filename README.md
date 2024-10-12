Sample weather forecast service to play with Rust

Get a PostgreSQL database running with Docker locally:
```bash
docker run -p 5432:5432 -e POSTGRES_USER=forecast -e POSTGRES_PASSWORD=forecast -e POSTGRES_DB=forecast -v `pwd`/init.sql:/docker-entrypoint-initdb.d/index.sql -d postgres
export DATABASE_URL="postgres://forecast:forecast@localhost:5432/forecast?sslmode=disable"
```

Reference: https://www.shuttle.dev/blog/2023/09/27/rust-vs-go-comparison
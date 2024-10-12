Ref: https://www.shuttle.dev/blog/2023/09/27/rust-vs-go-comparison

Get a PostgreSQL database running with Docker locally is:
```bash
docker run -p 5432:5432 -e POSTGRES_USER=forecast -e POSTGRES_PASSWORD=forecast -e POSTGRES_DB=forecast -v `pwd`/init.sql:/docker-entrypoint-initdb.d/index.sql -d postgres
export DATABASE_URL="postgres://forecast:forecast@localhost:5432/forecast?sslmode=disable"
FROM rust:latest AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:latest AS cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:latest AS builder
WORKDIR /app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

FROM gcr.io/distroless/cc
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
WORKDIR /app
COPY --from=builder /app/target/release/forecast .
EXPOSE 3000
CMD ["./forecast"]
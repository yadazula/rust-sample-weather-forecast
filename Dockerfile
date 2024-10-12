FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
WORKDIR /app
COPY --from=builder /app/target/release/forecast .
EXPOSE 3000
CMD ["./forecast"]
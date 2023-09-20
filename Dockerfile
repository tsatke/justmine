FROM rust:latest as builder
WORKDIR /app/src
COPY . .
RUN cargo build --release

FROM debian:stable-slim
WORKDIR /app
EXPOSE 25565 25565

COPY --from=builder /app/src/target/release/justmine /app
CMD ["/app/justmine"]
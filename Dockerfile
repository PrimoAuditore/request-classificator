FROM rust:slim as build

ENV META_TOKEN=""
ENV REDIS_URL=""

WORKDIR /app
COPY . .
RUN --mount=type=ssh cargo build --release

FROM debian:11-slim
WORKDIR /app
COPY --from=build /app/target/release/request-classificator ./request-classificator
CMD ["./request-classificator"]

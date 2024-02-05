FROM rust:1.74 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && groupadd -r appgroup && useradd -r -g appgroup appuser

WORKDIR /app
RUN chown appuser:appgroup /app

USER appuser
COPY --from=builder /usr/src/app/target/release/subc .

EXPOSE 3000
CMD ["/app/subc"]


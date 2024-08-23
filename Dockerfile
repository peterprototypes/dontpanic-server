FROM rust as builder

WORKDIR /app

COPY . .

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine as runtime

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/dontpanic-server .
COPY --from=builder /app/templates templates
COPY --from=builder /app/static static

CMD [ "/app/dontpanic-server" ]
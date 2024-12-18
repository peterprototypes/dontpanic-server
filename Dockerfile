FROM rust as rust_builder

WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM node as vite_builder

WORKDIR /app
COPY frontend/package.json frontend/package-lock.json ./
RUN npm install
COPY frontend/ .
RUN npm run build

FROM alpine as runtime

WORKDIR /app
COPY ./templates ./templates
COPY --from=rust_builder /app/target/x86_64-unknown-linux-musl/release/dontpanic-server .
COPY --from=vite_builder /app/dist frontend/dist

CMD [ "/app/dontpanic-server" ]
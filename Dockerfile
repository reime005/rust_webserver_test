FROM rust:1.47

WORKDIR /usr/src/rust_webserver_test
COPY . .

RUN cargo build --release --locked

RUN cargo install --path .

CMD ["rust_webserver_test"]

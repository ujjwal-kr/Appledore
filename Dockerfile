FROM rust:1.70-slim-buster AS builder

RUN mkdir /src
WORKDIR /src

COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:3.18.0
COPY --from=builder /src/target/x86_64-unknown-linux-musl/release/appledore /usr/local/bin/appledore
CMD ["appledore"]
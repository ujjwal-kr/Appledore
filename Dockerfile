FROM alpine:3.18.0
COPY ./target/x86_64-unknown-linux-musl/release/appledore ./appledore
EXPOSE 6379
CMD ["./appledore"]
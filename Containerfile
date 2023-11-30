# TODO

FROM --platform=arm64 debian:oldstable

COPY ./target/aarch64-unknown-linux-gnu/release/dive /usr/local/bin/
COPY ./external/*.so /usr/local/lib/

RUN ldconfig

CMD ["/usr/local/bin/dive"]

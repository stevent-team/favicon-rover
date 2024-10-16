FROM rust:latest

WORKDIR /build
COPY . .

RUN apt update && apt install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN cargo install --path . --features server && \
  mv /build/target/release/favicon-rover /favicon-rover && \
  # cleanup no longer needed build leftovers
  rm -rf /build /usr/local/rustup /usr/local/cargo

EXPOSE 3000
ENTRYPOINT ["/favicon-rover", "serve", "--host", "0.0.0.0"]

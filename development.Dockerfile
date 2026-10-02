FROM rust:latest AS development

RUN apt update && apt install -y curl && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch

# Same path as the sync targets of docker-compose.yml
WORKDIR /usr/src/elearning

# --- CACHE DES DÉPENDANCES ---
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
# Cette couche sera mise en cache tant que Cargo.toml ne change pas
RUN cargo build && rm -rf src
# -----------------------------

COPY src ./src
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 3006
# nosemgrep: dockerfile.security.missing-user.missing-user -- development image only (hot reload), the production Dockerfile runs as uid 65532
CMD ["/usr/local/bin/entrypoint.sh"]

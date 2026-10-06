# Same toolchain as the production `Dockerfile`, pinned by digest.
FROM rust:1.99-bookworm@sha256:fbc3a359627c6b5d9c8b20aae5c413a87392954f020006d7a9f7d95938964b23 AS development

RUN apt update && apt install -y curl && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch --locked

# Same path as the sync targets of docker-compose.yml
WORKDIR /usr/src/elearning

# --- DEPENDENCY CACHE ---
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
# This layer stays cached as long as Cargo.toml and Cargo.lock do not change
RUN cargo build --locked && rm -rf src
# -----------------------------

COPY src ./src
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 3006
# nosemgrep: dockerfile.security.missing-user.missing-user -- development image only (hot reload), the production Dockerfile runs as uid 65532
CMD ["/usr/local/bin/entrypoint.sh"]

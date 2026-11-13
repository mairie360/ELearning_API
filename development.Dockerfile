# Same toolchain as the production `Dockerfile`, pinned by digest.
FROM rust:1.100-trixie@sha256:e19840879045c951c905d0eaa9887f3439c62000e429963371b376bac8d38c31 AS development

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

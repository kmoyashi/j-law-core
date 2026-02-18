# =============================================================================
# j-law-core マルチステージ Dockerfile
#
# 各言語バインディングのテストをコンテナ上で実行するための環境。
# docker compose up test-all --build で全言語のテストを一括実行できる。
# =============================================================================

# ---- ベースステージ: Rust ツールチェイン + ソースコード ----
FROM rust:1.85-bookworm AS base-rust
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY tests/ tests/

# ---- Rust テスト ----
FROM base-rust AS test-rust
CMD ["cargo", "test", "-p", "j-law-core", "-p", "j-law-registry"]

# ---- Python テスト ----
FROM base-rust AS test-python
RUN apt-get update && apt-get install -y python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*
RUN pip3 install --break-system-packages maturin pytest
RUN maturin build -m crates/j-law-python/Cargo.toml \
    && pip3 install --break-system-packages target/wheels/*.whl
CMD ["pytest", "crates/j-law-python/tests/", "-v"]

# ---- WASM/JS テスト ----
FROM base-rust AS test-wasm
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://rustwasm.github.io/wasm-pack/installer/init.sh | sh
RUN wasm-pack build --target nodejs crates/j-law-wasm
CMD ["node", "--test", "crates/j-law-wasm/tests/real_estate.test.mjs", "crates/j-law-wasm/tests/income_tax.test.mjs"]

# ---- Ruby テスト ----
FROM base-rust AS test-ruby
RUN apt-get update && apt-get install -y ruby ruby-dev libclang-dev \
    && rm -rf /var/lib/apt/lists/*
RUN gem install bundler
WORKDIR /app/crates/j-law-ruby
RUN bundle install
RUN bundle exec rake compile
CMD ["bundle", "exec", "rake", "test"]

# ---- Go テスト ----
FROM base-rust AS test-go
ARG GO_VERSION=1.23.6
RUN curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-$(dpkg --print-architecture).tar.gz" \
    | tar -C /usr/local -xz
ENV PATH="/usr/local/go/bin:${PATH}"
WORKDIR /app
RUN cargo build -p j-law-cgo
WORKDIR /app/crates/j-law-go
CMD ["go", "test", "-v", "./..."]

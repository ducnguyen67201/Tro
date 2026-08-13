set dotenv-load := true

bootstrap:
    corepack enable
    pnpm install --frozen-lockfile
    cargo fetch --locked

dev:
    pnpm --filter @tro/desktop tauri dev

check:
    pnpm format:check
    pnpm lint
    pnpm typecheck
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    pnpm test --run
    cargo test --workspace --locked

build:
    cargo build --workspace --locked
    pnpm --filter @tro/desktop build

eval-offline:
    cargo run -p eval-runner -- --offline

eval-live:
    cargo run -p eval-runner -- --live

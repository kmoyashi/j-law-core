# J-Law-Core Makefile
#
# CIで実行される全チェックをローカルでワンコマンドで再現する。
# コードを変更したら必ず `make ci` を実行してからプッシュすること。

.PHONY: all fmt fmt-check clippy test audit check-versions ci docker-test

## デフォルト: CIチェック一式を実行
all: ci

## コードを自動フォーマットする
fmt:
	cargo fmt --all

## フォーマットチェック（CI と同等）
fmt-check:
	cargo fmt --all -- --check

## Clippy リント（CI と同等）
clippy:
	cargo clippy --workspace -- -D warnings

## Rust テストを実行する（CI と同等）
test:
	cargo test --workspace

## セキュリティ監査（cargo-audit が必要）
audit:
	cargo audit

## 公開パッケージのバージョン整合性を確認する
check-versions:
	./scripts/verify_release_versions.sh

## CIチェック一式: フォーマット・リント・テストを順番に実行する
##
## プッシュ前に必ずこのコマンドを実行すること。
## .github/workflows/ci.yml の lint + test-rust ジョブに相当する。
ci: check-versions fmt-check clippy test

## 全言語バインディングテストを Docker で実行する
## GitHub Actions では .github/workflows/ci.yml の言語別マトリクスで直接実行する
docker-test:
	docker compose up test-all --build

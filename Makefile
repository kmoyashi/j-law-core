# J-Law-Core Makefile
#
# CIで実行される全チェックをローカルでワンコマンドで再現する。
# コードを変更したら必ず `make ci` を実行してからプッシュすること。

.PHONY: all fmt fmt-check clippy test audit ci docker-test

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

## CIチェック一式: フォーマット・リント・テスト・全言語バインディングテストを順番に実行する
##
## プッシュ前に必ずこのコマンドを実行すること。
## .github/workflows/ci.yml の全ジョブに相当する。
## 実行時間: 5-10分程度（docker-test で時間がかかります）
ci: fmt-check clippy test docker-test

## 全言語バインディングテストを Docker で実行する（CI の test-bindings ジョブに相当）
docker-test:
	docker compose up test-all --build

# J-Law-Core Makefile
#
# CIで実行される全チェックをローカルでワンコマンドで再現する。
# コードを変更したら必ず `make ci` を実行してからプッシュすること。

.PHONY: all fmt fmt-check clippy test audit check-versions sync-go-native ci docker-test

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

## Go バインディング用の同梱ネイティブアーカイブをソースと同期する
##
## verify-native で差分がなければスキップし、差分があれば自動リビルドする。
## リビルドされた場合はコミットに含めること。
sync-go-native:
	@$(MAKE) -C crates/j-law-go verify-native 2>/dev/null \
		|| $(MAKE) -C crates/j-law-go sync-native

## CIチェック一式: フォーマット・リント・テスト・Go native同期を順番に実行する
##
## プッシュ前に必ずこのコマンドを実行すること。
## .github/workflows/ci.yml の lint + test-rust + go verify-native ジョブに相当する。
ci: check-versions fmt-check clippy test sync-go-native

## 全言語バインディングテストを Docker で実行する
## GitHub Actions では .github/workflows/ci.yml の言語別マトリクスで直接実行する
docker-test:
	docker compose up test-all --build

# J-Law-Core Makefile
#
# CIで実行される全チェックをローカルでワンコマンドで再現する。
# コードを変更したら必ず `make ci` を実行してからプッシュすること。

.PHONY: all fmt fmt-check clippy test audit check-versions bump-version sync-go-native sync-go-native-all ci docker-test

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

## 全公開パッケージのバージョンを一括更新する (例: make bump-version V=0.1.0)
bump-version:
	@test -n "$(V)" || (echo "usage: make bump-version V=<version>" >&2; exit 1)
	./scripts/bump_version.sh "$(V)"

## Go バインディング用の現在のプラットフォーム向け同梱ネイティブアーカイブをソースと同期する
##
## verify-native で差分がなければスキップし、差分があれば自動リビルドする。
## リビルドされた場合はコミットに含めること。
sync-go-native:
	@$(MAKE) -C crates/j-law-go verify-native 2>/dev/null \
		|| $(MAKE) -C crates/j-law-go sync-native

## Go バインディング用の全対応プラットフォーム分の同梱ネイティブアーカイブをソースと同期する
##
## verify-native-all で差分がなければスキップし、差分があれば自動リビルドする。
## リポジトリに同梱する配布物を更新するメンテナ向け。
sync-go-native-all:
	@$(MAKE) -C crates/j-law-go verify-native-all 2>/dev/null \
		|| $(MAKE) -C crates/j-law-go sync-native-all

## CIチェック一式: フォーマット・リント・テスト・Go native同期を順番に実行する
##
## プッシュ前に必ずこのコマンドを実行すること。
## ローカルでは現在のプラットフォーム向け Go 配布物を確認する。
ci: check-versions fmt-check clippy test sync-go-native

## 全言語バインディングテストを Docker で実行する
## GitHub Actions では .github/workflows/ci.yml の言語別マトリクスで直接実行する
docker-test:
	docker compose up test-all --build

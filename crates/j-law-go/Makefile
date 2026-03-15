# j-law-go Makefile
#
# Go パッケージは配布用の静的ライブラリを `native/` 配下に同梱する。
# 利用者は `go get` 後にそのまま利用でき、メンテナは Rust 側変更時のみ
# `make sync-native` で現在のプラットフォーム向け配布物を更新する。

WORKSPACE_ROOT := $(realpath $(dir $(abspath $(lastword $(MAKEFILE_LIST))))/../..)
CARGO         := cargo
GO            := go
SYNC_SCRIPT   := $(WORKSPACE_ROOT)/scripts/sync_go_native_archive.sh
HOST_GOOS     := $(shell $(GO) env GOOS)
HOST_GOARCH   := $(shell $(GO) env GOARCH)
HOST_PLATFORM := $(HOST_GOOS)_$(HOST_GOARCH)
NATIVE_ARCHIVE := native/$(HOST_PLATFORM)/libj_law_c_ffi.a

.PHONY: all ensure-native sync-native verify-native build-rust build-rust-release test test-fresh clean

## デフォルト: 同梱配布物を使って Go テスト
all: test

## 現在のプラットフォーム向け配布物が存在することを確認
ensure-native:
	@test -f $(NATIVE_ARCHIVE) || (echo "missing vendored native archive: $(NATIVE_ARCHIVE)"; \
		echo "Maintainers: run 'make sync-native' before running Go tests."; exit 1)

## 現在のプラットフォーム向け配布物を更新
sync-native:
	@$(SYNC_SCRIPT)

## 現在のプラットフォーム向け配布物が Rust ソースと同期しているか検証
verify-native:
	@tmp_dir=$$(mktemp -d); \
	trap 'rm -rf "$$tmp_dir"' EXIT; \
	JLAW_GO_NATIVE_DEST_ROOT="$$tmp_dir" $(SYNC_SCRIPT); \
	cmp -s "$$tmp_dir/$(HOST_PLATFORM)/libj_law_c_ffi.a" "$(NATIVE_ARCHIVE)" || \
		(echo "vendored native archive is stale: $(NATIVE_ARCHIVE)"; \
		echo "Run 'make sync-native' and commit the updated archive."; exit 1)

## 互換用エイリアス: 配布物を更新
build-rust: sync-native

## 互換用エイリアス: 配布物を更新
build-rust-release: sync-native

## 同梱配布物を使って Go テストを実行
test: ensure-native
	$(GO) test -v ./...

## 現在のプラットフォーム向け配布物を更新してから Go テストを実行
test-fresh: sync-native test

clean:
	$(GO) clean -testcache

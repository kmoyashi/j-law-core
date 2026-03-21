# j-law-go Makefile
#
# Go パッケージは配布用の静的ライブラリを `native/` 配下に同梱する。
# 利用者は `go get` 後にそのまま利用でき、メンテナは Rust 側変更時のみ
# `make sync-native` で現在のプラットフォーム向け配布物を更新する。

WORKSPACE_ROOT := $(realpath $(dir $(abspath $(lastword $(MAKEFILE_LIST))))/../..)
CARGO         := cargo
GO            := go
SYNC_SCRIPT   := $(WORKSPACE_ROOT)/scripts/sync_go_native_archive.sh
SUPPORTED_TARGETS := aarch64-apple-darwin x86_64-apple-darwin x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
HOST_GOOS     := $(shell $(GO) env GOOS)
HOST_GOARCH   := $(shell $(GO) env GOARCH)
HOST_PLATFORM := $(HOST_GOOS)_$(HOST_GOARCH)
NATIVE_ARCHIVE := native/$(HOST_PLATFORM)/libj_law_c_ffi.a

.PHONY: all ensure-native sync-native sync-native-all verify-native verify-native-all build-rust build-rust-release test test-fresh clean

## デフォルト: 同梱配布物を使って Go テスト
all: test

## 現在のプラットフォーム向け配布物が存在することを確認
ensure-native:
	@test -f $(NATIVE_ARCHIVE) || (echo "missing vendored native archive: $(NATIVE_ARCHIVE)"; \
		echo "Maintainers: run 'make sync-native' before running Go tests."; exit 1)

## 現在のプラットフォーム向け配布物を更新
sync-native:
	@$(SYNC_SCRIPT)

## 全対応プラットフォーム向け配布物を更新
sync-native-all:
	@for target in $(SUPPORTED_TARGETS); do \
		echo "syncing $$target"; \
		JLAW_GO_NATIVE_TARGET=$$target $(SYNC_SCRIPT); \
	done

## 現在のプラットフォーム向け配布物が Rust ソースと同期しているか検証
verify-native:
	@JLAW_GO_NATIVE_ACTION=verify $(SYNC_SCRIPT)

## 全対応プラットフォーム向け配布物が Rust ソースと同期しているか検証
verify-native-all:
	@for target in $(SUPPORTED_TARGETS); do \
		echo "verifying $$target"; \
		JLAW_GO_NATIVE_ACTION=verify JLAW_GO_NATIVE_TARGET=$$target $(SYNC_SCRIPT); \
	done

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

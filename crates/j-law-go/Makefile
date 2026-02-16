# j-law-go Makefile
#
# Go パッケージは j-law-cgo（Rust staticlib）に静的リンクしている。
# テスト・ビルドの前に必ず `make build-rust` を実行すること。

WORKSPACE_ROOT := $(realpath $(dir $(abspath $(lastword $(MAKEFILE_LIST))))/../..)
CARGO         := cargo
GO            := go

.PHONY: all build-rust test clean

## デフォルト: Rust ビルド → Go テスト
all: test

## Rust staticlib をビルドする（target/debug/libj_law_cgo.a を生成）
build-rust:
	$(CARGO) build -p j-law-cgo --manifest-path $(WORKSPACE_ROOT)/Cargo.toml

## Rust staticlib をリリースビルドする（target/release/libj_law_cgo.a を生成）
build-rust-release:
	$(CARGO) build -p j-law-cgo --release --manifest-path $(WORKSPACE_ROOT)/Cargo.toml

## Go テストを実行する（debug ビルドのライブラリを使用）
test: build-rust
	$(GO) test -v ./...

## Go テストをリリースビルドのライブラリで実行する
test-release: build-rust-release
	CGO_LDFLAGS="-L$(WORKSPACE_ROOT)/target/release" $(GO) test -v ./...

clean:
	$(GO) clean -testcache

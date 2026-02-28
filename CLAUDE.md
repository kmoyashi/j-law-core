# J-Law-Core

## 概要

日本の法律計算ライブラリ。Rustコアに対して多言語バインディング（Python/WASM/Ruby/Go/C）を提供。

## 実装ルール

### フォーマット・リント・ビルド

```bash
# フォーマット
cargo fmt --all

# リント
cargo clippy --all-targets --all-features -- -D warnings

# ビルド・テスト
cargo build --all-features
cargo test --all-features
```

### CIチェック

プッシュ前に `.github/workflows/ci.yml` の全チェックが通ることを確認。

## 問題解決の姿勢

目の前のエラーやビルド失敗だけでなく、**本質的な課題**を考える。

表面的な修正ではなく、根本原因を解決する実装を心がける。

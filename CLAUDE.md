# J-Law-Core

## 概要

日本の法律計算ライブラリ。Rustコアに対して多言語バインディング（Python/WASM/Ruby/Go/C）を提供。

## 実装ルール

アーキテクチャ・コーディング規約・コミットルールは [CONTRIBUTING.md](./CONTRIBUTING.md) を参照すること。

### フォーマット・リント・テスト

```bash
# コードを自動フォーマット
make fmt

# フォーマット・リント・テストを一括実行（プッシュ前に必ず実行）
make ci
```

個別に実行する場合:

```bash
make fmt-check   # フォーマットチェック
make clippy      # Clippy リント
make test        # Rust テスト
make docker-test # 全言語バインディングテスト（Docker）
```

### CIチェック

**コードを変更したら必ず以下を実行してからプッシュすること：**

```bash
# 1. Rust コードのフォーマット・リント・テスト
make ci

# 2. 全言語バインディングテスト（Docker必須）
make docker-test
```

- `make ci` は `.github/workflows/ci.yml` の lint + test-rust ジョブと同等のチェックを行う
- `make docker-test` は Python/Ruby/Go/WASM/C のバインディング全体をテストする
- Docker が起動していない場合は自動起動される

**Docker セットアップ（初回のみ）：**
- この環境では Docker デーモンをバックグラウンドで起動する必要があります
- セッション開始時に自動起動設定があるため、通常は手動操作不要です

## 問題解決の姿勢

目の前のエラーやビルド失敗だけでなく、**本質的な課題**を考える。

表面的な修正ではなく、根本原因を解決する実装を心がける。

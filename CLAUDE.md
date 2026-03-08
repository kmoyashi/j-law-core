# J-Law-Core

## 概要

日本の法律計算ライブラリ。Rustコアに対して多言語バインディング（Python/WASM/Ruby/Go/C）を提供。

## 実装ルール

アーキテクチャ・コーディング規約・コミットルールは [CONTRIBUTING.md](./CONTRIBUTING.md) を参照すること。

### フォーマット・リント・テスト

```bash
# コードを自動フォーマット
make fmt

# フォーマット・リント・Rustテスト・全言語バインディングテストを一括実行（プッシュ前に必ず実行）
# 実行時間: 5-10分程度
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

**コードを変更したら必ず `make ci` を実行してからプッシュすること。**

`make ci` は以下の処理を順番に実行します：

1. **fmt-check** - コードフォーマットチェック
2. **clippy** - Rust リント
3. **test** - Rust テスト
4. **docker-test** - Python/Ruby/Go/WASM/C 全言語バインディングテスト

これは `.github/workflows/ci.yml` の **全ジョブ**に相当します。

**所要時間:**
- 初回実行: 10-15分（Docker イメージビルド含む）
- 2回目以降: 5-10分（キャッシュ利用）

**Docker について：**
- Docker デーモンは セッション開始時に自動起動されます
- 手動操作は不要です

## 問題解決の姿勢

目の前のエラーやビルド失敗だけでなく、**本質的な課題**を考える。

表面的な修正ではなく、根本原因を解決する実装を心がける。

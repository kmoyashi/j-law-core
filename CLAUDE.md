# J-Law-Core

## 概要

日本の法律計算ライブラリ。Rustコアに対して多言語バインディング（Python/WASM/Ruby/Go/C）を提供。

## 実装ルール

アーキテクチャ・コーディング規約・コミットルールは [CONTRIBUTING.md](./CONTRIBUTING.md) を参照すること。

### フォーマット・リント・テスト

```bash
# コードを自動フォーマット
make fmt

# フォーマット・リント・テストを実行（プッシュ前に必ず実行）
# 実行時間: 3-5分程度
make ci
```

個別に実行する場合:

```bash
make fmt-check   # フォーマットチェック
make clippy      # Clippy リント
make test        # Rust テスト
make test-python # Python バインディングテスト（ローカル）
make docker-test # 全言語バインディングテスト（Docker経由）
```

### CIチェック

**コードを変更したら必ず `make ci` を実行してからプッシュすること。**

`make ci` は以下の処理を順番に実行します：

1. **fmt-check** - コードフォーマットチェック
2. **clippy** - Rust リント
3. **test** - Rust テスト
4. **test-python** - Python バインディングテスト（ホスト上で実行）

**所要時間:** 3-5分程度

### 全言語バインディングテスト

Ruby/Go/WASM などの他言語テストは **GitHub Actions で実行** されます：

```bash
# GitHub Actions と同等の全テストを実行（オプション）
make docker-test
```

- 環境: Docker 経由で全言語ビルド・テスト実行
- ローカルマシン・GitHub Actions の両方で実行可能
- 実行時間: 10-15分（初回）/ 5-10分（以降）

**Docker について：**
- Docker デーモンは セッション開始時に自動起動されます
- 手動操作は不要です

## 問題解決の姿勢

目の前のエラーやビルド失敗だけでなく、**本質的な課題**を考える。

表面的な修正ではなく、根本原因を解決する実装を心がける。

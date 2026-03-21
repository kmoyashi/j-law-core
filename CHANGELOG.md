# Changelog

このプロジェクトのすべての注目すべき変更はこのファイルに記録されます。

形式は [Keep a Changelog](https://keepachangelog.com/ja/1.0.0/) に基づき、
バージョン管理は [Semantic Versioning](https://semver.org/lang/ja/) に準拠します。

## [Unreleased]

## [0.0.1] - 2026-03-21

### Added

#### ドメイン実装

- **消費税**（消費税法 第29条）
  - 標準税率（10%）・軽減税率（8%）の税額計算
  - 税込価格・税抜価格の相互変換
  - 適用税率の返却

- **不動産媒介報酬**（宅地建物取引業法 第46条）
  - 売買価格に基づく3段階ティア計算
  - 低廉な空き家等の特例対応
  - 消費税との連携計算

- **所得税**（所得税法 第89条 / 復興財源確保法 第13条）
  - 速算表による所得税額計算
  - 復興特別所得税の計算
  - 所得控除の適用
  - 通し計算（所得控除 → 課税所得 → 税額）

- **所得控除**（所得税法 第72条〜第87条）
  - 基礎控除・配偶者控除・扶養控除等の各種所得控除

- **印紙税**（印紙税法 別表第一）
  - 主要文書コードの税額算出
  - 軽減措置の適用
  - 非課税フラグ・適用ルールの返却

- **源泉徴収**（所得税法 第204条第1項）
  - 報酬・料金等の二段階税率類型
  - 応募作品賞金の免税判定
  - 区分消費税控除

#### 言語バインディング

- **Python バインディング** (`j-law-python`)
  - `ctypes` 経由で C ABI を利用
  - CPython 3.10〜3.14 対応
  - PyPI 配布向けのホイールビルド対応

- **WASM バインディング** (`j-law-wasm`)
  - `wasm-bindgen` ベースの JavaScript / WebAssembly バインディング
  - `wasm-pack --target nodejs` によるパッケージ生成
  - Node.js 20〜25 対応

- **Ruby バインディング** (`j-law-ruby`)
  - `ffi` gem 経由で C ABI を利用
  - Ruby 3.1〜4.0 対応
  - ソース gem・バイナリ gem の両形式に対応

- **Go バインディング** (`j-law-go`)
  - CGo 経由で C ABI を利用
  - Go 1.21+ 対応
  - `darwin/amd64` `darwin/arm64` `linux/amd64` `linux/arm64` 向け同梱 native archive

- **C ABI** (`j-law-c-ffi`)
  - 静的ライブラリ (`.a`) および動的ライブラリ (`.so` / `.dylib`) の生成

#### インフラ・ツール

- Rust コアの整数演算・分数演算による金額計算（`f64` / `f32` 不使用）
- 法令パラメータ JSON registry による施行日管理
- `tests/fixtures/` 共通 JSON による多言語クロステスト
- GitHub Actions CI（lint / Rust テスト / セキュリティ監査 / 各言語バインディングテスト）
- Go モジュール公開ワークフロー（GitHub Release トリガー）

[Unreleased]: https://github.com/kmoyashi/j-law-core/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/kmoyashi/j-law-core/releases/tag/v0.0.1

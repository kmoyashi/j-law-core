# Changelog

このプロジェクトのすべての注目すべき変更はこのファイルに記録されます。

形式は [Keep a Changelog](https://keepachangelog.com/ja/1.0.0/) に基づき、
バージョン管理は [Semantic Versioning](https://semver.org/lang/ja/) に準拠します。

## [Unreleased]

### Fixed

- **所得控除が2025年分以後の税制改正を反映していなかった問題を修正**
  - 2025年分以後も基礎控除48万円・配偶者の所得要件48万円以下で計算していた
  - 令和7年度改正（令和7年法律第13号）: 2025年分の基礎控除（最大95万円）、配偶者・扶養親族の所得要件58万円以下
  - 令和8年度改正（令和8年法律第12号）: 2026・2027年分の基礎控除（最大104万円）、2028年分以後の基礎控除（最大99万円）、所得要件62万円以下
- **Go 1.26.11 以降でビルドできない問題を修正**
  - 同梱の静的ライブラリに含まれる LTO 用セクション（Rust 標準ライブラリ由来の `.llvmbc` / `__LLVM,__bitcode` など）を cgo が拒否していた
  - `scripts/sync_go_native_archive.sh` で生成後に `llvm-objcopy` でこれらのセクションを除去し、残存しないことを検証する（アーカイブサイズも約 35〜55% 縮小）
- CI / 公開ワークフローで rustfmt・clippy を事前にインストールし、ビルド途中の rustup による自動インストール（Python バインディングのビルドで競合して失敗していた）を避ける

### Added

- **特定親族特別控除**（所得税法 第84条の2、2025年分以後）
  - Rust: `SpecificRelativeDeductionInput` / `SpecificRelativeDeductionParams` / `IncomeDeductionKind::SpecificRelative`
  - 全言語バインディングに特定親族の入力を追加（Python `specific_relatives` / WASM `specificRelatives` / Ruby `specific_relatives:` / Go `SpecificRelatives`）
- 所得控除 registry の区分テーブル（基礎控除・配偶者控除・特定親族特別控除）の連続性検証と `RegistryError::InvalidBracketTable`

### Changed

- **Breaking**: C FFI バージョンを 5 に更新。`JLawIncomeDeductionInput` の末尾に `specific_relative_count` / `specific_relative_total_income_amounts[J_LAW_MAX_SPECIFIC_RELATIVES]` を追加
- **Breaking (Rust)**: `PersonalDeductionParams` に `specific_relative`、`PersonalDeductionInput` に `specific_relatives` フィールドを追加
- 2025年分以後の所得控除内訳に「特定親族特別控除」行（kind = 8）を扶養控除の直後に追加（2024年分以前の内訳は変更なし）

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

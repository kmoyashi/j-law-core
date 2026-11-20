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
- **復興特別所得税の課税期間延長（2037年→2047年）を反映**（令和8年法律第12号）
  - 所得税: 2038〜2047年分で復興特別所得税が課されていなかった（ポリシーが `2013..=2037` を直書きしていた）
  - 源泉徴収: 2038年以後の基準日で `DateOutOfRange` エラーになっていた

### Added

- **防衛特別所得税**（防衛財源確保法 第5条の9、2027年分以後）
  - 所得税: 2027〜2047年分は復興特別所得税 1.1% + 防衛特別所得税 1%（合計 2.1% は不変）、2048年分以後は防衛特別所得税 1% のみ
  - 付加税は合計額で1円未満を切り捨て、防衛特別所得税額（1円未満切り捨て）を差し引いた残りを復興特別所得税額とする（防衛財源確保法 第5条の22第2項に基づき申告納税額は合計額で端数計算）
  - 結果に `defense_tax` / `defense_tax_applied` を追加（Python・Ruby・Go・WASM・C FFI）
  - 源泉徴収: registry を 2013〜2026年 / 2027〜2047年 / 2048年以後 に分割（合計税率は 2047年まで 10.21% / 20.42%、2048年以後 10.1% / 20.2%）

- **特定親族特別控除**（所得税法 第84条の2、2025年分以後）
  - Rust: `SpecificRelativeDeductionInput` / `SpecificRelativeDeductionParams` / `IncomeDeductionKind::SpecificRelative`
  - 全言語バインディングに特定親族の入力を追加（Python `specific_relatives` / WASM `specificRelatives` / Ruby `specific_relatives:` / Go `SpecificRelatives`）
- 所得控除 registry の区分テーブル（基礎控除・配偶者控除・特定親族特別控除）の連続性検証と `RegistryError::InvalidBracketTable`

### Changed

- **Breaking**: C FFI バージョンを 5 に更新。`JLawIncomeDeductionInput` の末尾に `specific_relative_count` / `specific_relative_total_income_amounts[J_LAW_MAX_SPECIFIC_RELATIVES]` を追加
- **Breaking (Rust)**: `PersonalDeductionParams` に `specific_relative`、`PersonalDeductionInput` に `specific_relatives` フィールドを追加
- **Breaking (Rust)**: `IncomeDeductionKind::SpecificRelative` と `RegistryError::InvalidBracketTable` を追加（これらの enum を網羅的に `match` しているコードは更新が必要）
- 2025年分以後の所得控除内訳に「特定親族特別控除」行（kind = 8）を扶養控除の直後に追加（2024年分以前の内訳は変更なし）。社会保険料控除以降の行の位置が1つずれるため、内訳は `kind` で参照すること
- C FFI: 所得控除の内訳が `J_LAW_MAX_DEDUCTION_LINES` を超える場合、切り捨てずにエラーを返す
- **Breaking**: C FFI バージョンを 6 に更新。`JLawIncomeTaxResult` / `JLawIncomeTaxAssessmentResult` の末尾に `defense_tax` / `defense_tax_applied` を追加
- **Breaking (Rust)**: `IncomeTaxParams` に `defense_tax`、`IncomeTaxResult` に `defense_tax` / `defense_tax_applied` フィールドを追加
- `IncomeTaxFlag::ApplyReconstructionTax`（各言語の `apply_reconstruction_tax`）は「付加税（復興特別所得税・防衛特別所得税）を適用する」の意味に変更。付加税の課税期間・税率は registry で判定し、`StandardIncomeTaxPolicy` は年を判定しない
- 付加税パラメータがない年分でフラグを指定した場合、エラーではなく付加税 0 円として扱う

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

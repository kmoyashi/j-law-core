# コントリビューションガイド

J-Law-Core へのコントリビューションを歓迎します。
このドキュメントでは、プロジェクトのアーキテクチャと実装ルールを説明します。

---

## 目次

- [開発環境のセットアップ](#開発環境のセットアップ)
- [アーキテクチャ概要](#アーキテクチャ概要)
- [型システムのルール](#型システムのルール)
- [エラー型の3層構造](#エラー型の3層構造)
- [Policy trait パターン](#policy-trait-パターン)
- [フラグの設計](#フラグの設計)
- [Rustdoc のルール](#rustdoc-のルール)
- [テストのルール](#テストのルール)
- [依存関係のルール](#依存関係のルール)
- [unsafe コードのルール](#unsafe-コードのルール)
- [言語バインディングの規約](#言語バインディングの規約)
- [コミットのルール](#コミットのルール)

---

## 開発環境のセットアップ

### 必要なツール

- Rust 1.94.0 toolchain（`rustup` 経由でインストール）
- `cargo fmt` / `cargo clippy`（Rust 1.94.0 toolchain に同梱）

### ビルドとテスト

```bash
# コードを自動フォーマット
make fmt

# フォーマット・リント・テストを一括実行
make ci
```

個別に実行する場合:

```bash
make fmt-check   # フォーマットチェック
make clippy      # Clippy リント
make test        # Rust テスト
```

### プッシュ前チェック

**プッシュ前に必ず `make ci` を実行し、全チェックが通ることを確認してください。**

```bash
make ci
```

`make ci` は以下を順番に実行します:
1. `cargo fmt --all -- --check` — フォーマットチェック
2. `cargo clippy --workspace -- -D warnings` — Clippy リント
3. `cargo test --workspace` — Rust テスト

---

## アーキテクチャ概要

### モジュール構成

```
crates/
├── j-law-core/         コアライブラリ（唯一の外部依存: thiserror）
│   └── src/
│       ├── lib.rs              公開 re-export のみ（ロジックなし）
│       ├── error.rs            全エラー型
│       ├── types/              共通型（FinalAmount, Rate, RoundingStrategy, LegalDate 等）
│       └── domains/            ドメインごとの計算ロジック
│           ├── consumption_tax/ 消費税（消費税法 第29条）
│           ├── income_tax/     所得税（所得税法 第89条）
│           ├── real_estate/    不動産（宅建業法 第46条）
│           └── stamp_tax/      印紙税（印紙税法 別表第一）
├── j-law-registry/     法令パラメータ管理（JSON → Rust 型のローダ）
├── j-law-python/       Python バインディング（ctypes + C ABI）
├── j-law-wasm/         WASM バインディング（wasm-bindgen）
├── j-law-ruby/         Ruby バインディング（ffi + C ABI）
├── j-law-c-ffi/        C ABI
└── j-law-go/           Go バインディング（CGo、非 workspace メンバー）
```

### ドメインの4ファイルパターン

全ドメインは以下の4ファイル構成に統一しています。新規ドメイン追加時もこのパターンを踏襲してください。

```
domains/<domain_name>/
├── mod.rs          サブモジュールの宣言 + pub use による re-export
├── params.rs       法令パラメータ型（Registry JSON からの読み込み対象）
├── context.rs      計算コンテキスト（入力値 + フラグ + ポリシー）
├── policy.rs       Policy trait 定義 + 標準実装（Standard*Policy）
└── calculator.rs   calculate_xxx() 関数 + 結果型
```

### mod.rs の re-export ルール

`mod.rs` では、そのドメインの主要な公開型を全て re-export します。
呼び出し元が `calculator::` や `context::` まで辿らなくても使えるようにするためです。

```rust
// 正しい例（income_tax/mod.rs）
pub use calculator::{calculate_income_tax, IncomeTaxResult, IncomeTaxStep};
pub use context::{IncomeTaxContext, IncomeTaxFlag};
pub use params::{IncomeTaxBracket, IncomeTaxParams, ReconstructionTaxParams};
pub use policy::StandardIncomeTaxPolicy;
```

re-export すべき項目:
- `calculator` の計算関数と結果型
- `context` のコンテキスト型とフラグ enum
- `params` のパラメータ型
- `policy` の標準ポリシー実装

---

## 型システムのルール

### 金額計算は整数のみ（float 禁止）

法令計算では端数処理の順序が結果を変えるため、**`f64` / `f32` の使用を全面禁止**しています。
`clippy.toml` の `disallowed-types` に設定されており、CI で自動的に検出されます。

```rust
// NG
let fee: f64 = price as f64 * 0.05;

// OK — 整数分数で計算
let rate = Rate { numer: 5, denom: 100 };
let amount = IntermediateAmount::from_exact(price);
let result = rate.apply(&amount, MultiplyOrder::MultiplyFirst, RoundingStrategy::Floor)?;
```

### 2つの金額型を正しく使い分ける

| 型 | 用途 | 説明 |
|---|---|---|
| `IntermediateAmount` | 計算途中 | 整数部 + 分数部（`whole + numer/denom`）で端数を保持 |
| `FinalAmount` | 最終結果 | 円単位の確定値。`finalize()` 経由でのみ生成 |

- 計算途中は必ず `IntermediateAmount` を使う
- `FinalAmount` への変換は `finalize(rounding)` のみ。直接 `FinalAmount::new()` は合算済みの確定値にのみ使う
- ドメイン固有の金額型を新設しない

### 乗算順序（MultiplyOrder）を明示する

`Rate::apply()` は `MultiplyOrder` を引数に取ります。
順序によって結果が変わるため、法令の端数処理に合った順序を明示的に選択してください。

- `MultiplyFirst`: 先に分子を掛けてから分母で割る（精度優先）
- `DivideFirst`: 先に分母で割ってから分子を掛ける（オーバーフロー回避）

---

## エラー型の3層構造

```
JLawError（トップレベル、#[from] で自動変換）
├── RegistryError      法令データの不整合（起動時・panic 許容層）
├── InputError         呼び出し元の入力不正（実行時）
└── CalculationError   計算中の異常（オーバーフロー等）
```

### ルール

- プロダクションコード（`crates/j-law-core/src/`）では **`unwrap()` / `expect()` / `panic!` 禁止**。全て `Result<T, E>` で返す。`clippy.toml` の `disallowed-methods` / `disallowed-macros` で CI 違反になる
- テストコードは `#[allow(clippy::disallowed_methods)]` で例外とする
- 新しいエラーバリアントは既存3種のいずれかに追加する。既存バリアントの削除・改名は Breaking Change
- エラーメッセージは日本語で記述する（ドメインの性質上、利用者は日本語話者を想定）

---

## Policy trait パターン

各ドメインは端数処理・特例適用の判定ロジックを trait で抽象化しています。

```rust
// policy.rs で trait を定義
pub trait IncomeTaxPolicy: std::fmt::Debug {
    fn should_apply_reconstruction_tax(&self, target_year: u16, flags: &HashSet<IncomeTaxFlag>) -> bool;
    fn tax_rounding(&self) -> RoundingStrategy;
    fn reconstruction_tax_rounding(&self) -> RoundingStrategy;
}

// 標準実装を同ファイルに定義
#[derive(Debug, Clone, Copy)]
pub struct StandardIncomeTaxPolicy;
```

### ルール

- Context は `Box<dyn Policy>` でポリシーを保持する（テスト時の差し替えを可能にするため）
- 標準ポリシーはゼロサイズ型（`struct StandardXxxPolicy;`）で実装する
- 特例の適用可否はポリシーが判定するが、**事実認定**（その取引が特例に該当するか）はライブラリの責任範囲外。フラグの docコメントに `WARNING` を付けること

---

## フラグの設計

法令の適用条件は `HashSet<DomainFlag>` で表現します。ブーリアン引数を増やさないでください。

```rust
// NG — 引数が増え続ける
fn calc(price: u64, is_low_cost: bool, is_reduced: bool, ...) -> ...

// OK — フラグセットで拡張可能
fn calc(ctx: &RealEstateContext) -> ...
// ctx.flags: HashSet<RealEstateFlag>
```

---

## Rustdoc のルール

### 公開 API には法的根拠を明記する

```rust
/// 宅建業法第46条に基づく媒介報酬を計算する。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
/// 国土交通省告示（2024年7月1日施行 / 2019年10月1日施行）
///
/// # 計算手順
/// 1. 各ティアの対象金額を求め、個別に切り捨てる
/// 2. ...
pub fn calculate_brokerage_fee(...) -> Result<CalculationResult, JLawError> {
```

必須セクション:
- **`# 法的根拠`** — 条文番号を具体的に記載（「所得税法 第89条第1項」等）
- **`# 計算手順`** — アルゴリズムを日本語で箇条書き

---

## 日付型のルール

### フィクスチャ JSON

クロス言語テストで共有する `tests/fixtures/<domain>.json` の日付フィールドは、
**ISO 8601 文字列（`"YYYY-MM-DD"`）** を使用してください。
レジストリ JSON の `effective_from` / `effective_until` と同じ形式で統一します。

```jsonc
// NG — 言語固有の数値フィールドを直接埋め込む
{ "year": 2024, "month": 8, "day": 1 }

// OK — ISO 8601 文字列で記述
{ "date": "2024-08-01" }
```

各言語のテストコードでは、日付文字列を各言語のネイティブ日付型に変換してからバインディング関数に渡します。

```python
# Python — datetime.date.fromisoformat() を使う
date = datetime.date.fromisoformat(inp["date"])
```

```javascript
// JavaScript — Date.UTC() を使う（JST 解釈・タイムゾーン非依存）
const [y, m, d] = c.input.date.split("-").map(Number);
const date = new Date(Date.UTC(y, m - 1, d));
```

```ruby
# Ruby — Date.parse() を使う
date = Date.parse(inp["date"])
```

```go
// Go — time.Parse() を使う
date, _ := time.Parse("2006-01-02", tc.Input.Date)
```

### バインディング関数シグネチャ

Rust コアと C ABI では、日付は `year: u16, month: u8, day: u8` の **3 引数** で受け取ります。
各言語バインディング（Python / WASM / Ruby / Go）では、各言語のネイティブ日付型を使用します:

- **Python**: `datetime.date`
- **JavaScript (WASM)**: `Date`
- **Ruby**: `Date`
- **Go**: `time.Time`

バインディング層でネイティブ日付型から year / month / day を抽出し、Rust コアに渡します。

---

## テストのルール

### テスト階層

| 階層 | 配置 | 対象 |
|---|---|---|
| ユニットテスト | 各 `src/*.rs` 内の `#[cfg(test)]` | 関数・型単位 |
| 統合テスト | `crates/j-law-core/tests/<domain>/` | ドメイン全体（Registry 読み込み含む） |
| バインディングテスト | 各 `crates/j-law-*/tests/` | FFI 経由の動作 |

### 統合テストの構成

```
tests/
├── income_tax.rs                        ← エントリファイル（自動検出対象）
├── income_tax/
│   ├── calculation_examples.rs          ← 公式計算例
│   └── edge_cases.rs                    ← 境界値テスト
├── real_estate.rs
├── real_estate/
│   ├── mlitt_examples.rs
│   └── edge_cases.rs
└── stamp_tax/
    └── calculation_examples.rs
```

### テスト品質の基準

- **公式資料に基づく**: テストケースは国税庁（NTA）・国交省（MLITT）の公式計算例をソースとする
- **手計算コメント必須**: テスト関数には期待値の手計算過程をコメントで残す

```rust
/// 課税所得 5,000,000円
/// 基準税額: 5,000,000 × 20% - 427,500 = 572,500
/// 復興税: 572,500 × 21/1000 = 12,022
/// 合計: 572,500 + 12,022 = 584,522 → 584,500（100円未満切捨）
```

- テストを削除したり `#[ignore]` で無効化してはならない
- 計算結果が1円ずれた場合は、期待値を書き換えるのではなく `breakdown` の各ステップの中間値でどこで差が出るか特定すること

---

## 依存関係のルール

### コアライブラリは最小依存

`j-law-core` の外部依存は **`thiserror` のみ**です。
組み込み先のバイナリサイズを抑えるため、安易に依存を追加しないでください。

### 循環依存の回避

```
j-law-core  ←（依存）←  j-law-registry
```

- `j-law-core` は `j-law-registry` に**依存しない**（循環になる）
- パラメータ型（`BrokerageFeeParams` 等）は `j-law-core` 側で定義する
- `j-law-registry` は `j-law-core` の `[dev-dependencies]` にのみ記載（統合テスト用）

### Registry JSON は整数のみ

```jsonc
// NG
{ "rate": 0.05 }

// OK
{ "rate_numer": 5, "rate_denom": 100 }
```

---

## unsafe コードのルール

- unsafe は **`j-law-c-ffi` の C ABI 境界のみ** に限定する
- unsafe 関数には `# Safety` セクションで前提条件（ポインタの非 null、バッファ長等）を文書化する
- コアライブラリ (`j-law-core`) に unsafe を追加しない

---

## 言語バインディングの規約

### 命名規則

| 言語 | 関数名 | プロパティ名 |
|---|---|---|
| Rust | `calculate_brokerage_fee` | `total_with_tax` |
| Python | `calc_brokerage_fee` | `total_with_tax` |
| JavaScript | `calcBrokerageFee` | `totalWithTax` |
| Ruby | `calc_brokerage_fee` | `total_with_tax` |
| Go | `CalcBrokerageFee` | `TotalWithTax` |
| C | `j_law_calc_brokerage_fee` | `total_with_tax` |

各言語の慣例に従いつつ、意味は統一してください。

### 新ドメイン追加時の対応

Rust コアにドメインを追加した場合、全バインディング（Python / WASM / Ruby / Go / C）に対応する関数を追加してください。

---

## コミットのルール

### コミットメッセージ

接頭辞を使い分けてください:

| 接頭辞 | 用途 | 例 |
|---|---|---|
| `feat:` | 新機能・新ドメイン追加 | `feat: 印紙税ドメインを追加` |
| `fix:` | バグ修正 | `fix: 低廉特例の最低保証額判定を修正` |
| `refactor:` | 動作を変えないコード改善 | `refactor: re-export を3ドメインで統一` |
| `test:` | テストの追加・修正 | `test: 所得税の境界値テストを追加` |
| `docs:` | ドキュメントのみの変更 | `docs: 印紙税の法的根拠を追記` |
| `ci:` | CI 設定の変更 | `ci: clippy の警告レベルを変更` |

- 本文は日本語で簡潔に（1行目は50文字以内を目安）
- 法令の条文番号や施行日に触れる変更は、コミットメッセージにも明記する

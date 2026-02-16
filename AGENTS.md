# AGENTS.md — J-Law-Core AIエージェント向け指示書

このファイルはClaude CodeなどのAIコーディングエージェントが読む指示書です。
コードを生成・編集する前に必ずこのファイルの内容を確認してください。

---

## プロジェクト概要（エージェント向け）

- **目的**: 日本の法令・告示・省令が定める各種計算を、法的正確性を保証して実装するライブラリ
- **アーキテクチャ**: ドメイン単位で法令を追加できる設計。不動産（`real_estate`）はPhase 1の参照実装
- **Cargo workspace**: `crates/j-law-core`（共通基盤）/ `crates/j-law-registry`（法令パラメータ）/ `crates/j-law-python`（PyO3）
- **主な依存クレート**: `thiserror = "1"`, `serde/serde_json = "1"`, `pyo3 = "0.21"`（j-law-pythonのみ）
- **Rustエディション**: 2021

---

## セッション開始時の必須手順

**セッションをまたいで記憶は引き継がれません。毎回以下を実行してください。**

```sh
cat Cargo.toml
ls crates/j-law-core/src/domains/
cargo test --all 2>&1 | tail -20
```

現状のドメイン構成とテスト状態を確認してから作業を開始すること。

---

## コーディングルール（違反するとCIが失敗します）

### ルール1: 金額・数値計算に `f64` / `f32` を使用禁止

法令計算は端数処理の順序が結果を左右します。浮動小数点は使用禁止です。
`crates/j-law-core/src/` への違反はCIで検出・拒否されます。

```rust
// NG — 法令計算で絶対に書かない
let fee: f64 = price as f64 * 0.05;

// OK — IntermediateAmount + Rate の整数演算を使う
let fee = IntermediateAmount::from_exact(price)
    .apply_rate(&Rate { numer: 5, denom: 100 }, RoundingStrategy::Floor);
```

確認コマンド: `grep -r 'f64\|f32' crates/j-law-core/src/`（結果が出たらNG）

### ルール2: `panic!` 使用禁止（コア層）

`crates/j-law-core/src/` 内では `panic!` / `unwrap()` / `expect()` を使わず、すべて `Result<T, E>` で返すこと。
Registry層（`crates/j-law-registry/src/`）の起動時バリデーションのみ `panic!` を許容します。

```rust
// NG
fn try_new(denom: u64) -> Self {
    if denom == 0 { panic!("zero denominator"); }
    Self(denom)
}

// OK
fn try_new(denom: u64) -> Result<Self, InputError> {
    if denom == 0 { return Err(InputError::ZeroDenominator); }
    Ok(Self(denom))
}
```

確認コマンド: `grep -rn 'panic!\|\.unwrap()\|\.expect(' crates/j-law-core/src/`

### ルール3: Registry JSONの数値は整数のみ

`crates/j-law-registry/data/` 内のJSONに小数点を含む数値を書いてはいけません。

```json
// NG
{ "rate": 0.05 }

// OK
{ "rate": { "numer": 5, "denom": 100 } }
```

確認コマンド: `grep -rn '\.' crates/j-law-registry/data/`（文字列・パス以外の箇所に小数点があればNG）

### ルール4: `pub` な型・関数には根拠条文を docコメントで明記

```rust
/// 標準媒介報酬の3段階ティア計算を実行する。
///
/// # 法的根拠
/// 宅地建物取引業法 第46条第1項
/// 国土交通省告示（2024年7月1日施行）
pub fn calculate_brokerage_fee(
    ctx: &RealEstateContext,
    params: &BrokerageFeeParams,
) -> Result<CalculationResult, JLawError> {
```

新しいドメインを追加する場合も同様に、該当条文・告示名・施行日を必ず記載してください。

### ルール5: TDD（テストファースト）

実装前にテストを書くこと。`cargo test` がグリーンになることが各タスクの完了基準です。
テストを削除・`#[ignore]` で誤魔化してはいけません。

---

## 型システム（必ず従うこと）

### 金額型（全ドメイン共通）

| 型 | 用途 | 備考 |
|---|---|---|
| `FinalAmount(u64)` | 最終的な円単位の金額 | 税込/税抜/税額など最終値 |
| `IntermediateAmount { whole, numer, denom }` | 計算途中の分数表現 | 端数処理前の中間値 |

`FinalAmount` と `IntermediateAmount` は用途を混在させないこと。
どのドメインでもこの2型を使い、ドメイン固有の金額型を新設してはいけません。

### エラー型階層（全ドメイン共通）

```
JLawError
├── Registry(RegistryError)       — 法令パラメータデータの不整合
│   ├── PeriodOverlap { domain, from, until }
│   ├── PeriodGap { domain, end, next_start }
│   ├── FloatProhibited { path }
│   └── ZeroDenominator { path }
├── Input(InputError)             — ユーザー入力の不正
│   ├── NegativeAmount { value }
│   ├── DateOutOfRange { date }
│   └── ConflictingFlags { flag_a, flag_b }
└── Calculation(CalculationError) — 計算時の異常
    ├── Overflow { step }
    └── PolicyNotApplicable { reason }
```

- 新しいエラーを追加するときは既存バリアントを**削除・改名しない**こと（Breaking Change になります）
- ドメイン固有のエラーは上記4種のいずれかの傘下に追加してください

### ドメイン固有フラグ

各ドメインは独自の `<Domain>Flag` enumを持ちます。フラグの事実認定（「この取引は○○に該当するか」の判断）はライブラリの責任範囲外です。事実認定を要するフラグのdocコメントには必ず以下の警告を付けてください:

```rust
/// WARNING: このフラグの事実認定はライブラリの責任範囲外です。
/// 呼び出し元が正しく判断した上で指定してください。
```

---

## ドメインの追加ルール

新しい法令ドメインを追加する場合は以下の構成に従ってください。

### ディレクトリ構成

```
crates/j-law-core/src/domains/<domain_name>/
├── mod.rs          # pub use でサブモジュールを再エクスポート
├── context.rs      # 計算コンテキスト（入力値 + フラグ + ポリシー）
├── policy.rs       # <Domain>Policy trait + 標準実装
└── calculator.rs   # calculate_xxx() 関数

crates/j-law-registry/data/<domain_name>/
└── <law_name>.json  # 法令パラメータ（整数のみ・period管理）

tests/<domain_name>/
├── official_examples.rs  # 公式計算例テスト（出典コメント必須）
└── edge_cases.rs         # 境界値テスト
```

### 必須実装項目

各ドメインは以下を必ず実装してください:

1. **Contextオブジェクト**: 入力値・target_date・フラグのセット
2. **Policy trait**: 端数処理戦略・特例適用判定などのカスタマイズポイント
3. **標準Policy実装**: 省庁の標準解釈に従ったデフォルト実装
4. **calculate関数**: `Result<CalculationResult, JLawError>` を返す主計算関数
5. **CalculationResult**: `total_with_tax`・`total_without_tax`・`breakdown` を含む結果型
6. **公式計算例テスト**: 省庁や公的機関が公表している計算例を全件テスト化

### Registry JSONの必須フィールド

```json
{
  "domain": "<domain_name>",
  "history": [
    {
      "effective_from": "YYYY-MM-DD",
      "effective_until": "YYYY-MM-DD",  // 現行版は null
      "status": "active",               // または "superseded"
      "citation": {
        "law_name": "法令名",
        "article": 46,
        "paragraph": 1
      },
      "params": {
        // 全数値は整数または { "numer": N, "denom": N } 形式
      }
    }
  ]
}
```

---

## 計算仕様 — 不動産ドメイン（Phase 1 参照実装）

### 3段階ティア計算

| 売買価格の範囲 | 率 |
|---|---|
| 200万円以下の部分 | 5/100 |
| 200万円超〜400万円以下の部分 | 4/100 |
| 400万円超の部分 | 3/100 |

**端数処理の順序（厳守）**:
1. 各ティアの金額を個別に `RoundingStrategy::Floor` で切り捨て
2. 切り捨て済みの整数を合計
3. 合計に消費税 10/100 を乗じ `Floor` で切り捨て

各ティアを合計してから一括で端数処理するのは**誤り**です。

### 低廉な空き家特例（2024年7月1日〜）

- 適用条件: `price <= 8_000_000` かつ `flags.contains(RealEstateFlag::IsLowCostVacantHouse)`
- 特例は**最低保証額（floor）**であり、上限キャップ（ceiling）ではありません。通常計算が `330_000` を下回る場合に `330_000` まで**引き上げ**ます。
  - `subtotal = subtotal.max(special.fee_ceiling_exclusive_tax)` — これが正しい実装です
  - `if subtotal > ceiling { subtotal = ceiling }` — **誤りです**（通常計算 > 特例上限の場合に値を下げてしまう）
- 引き上げ後に消費税 10/100（Floor）を加算

**8,000,000円の計算例（フラグあり）**:
- 通常計算: tier1(100,000) + tier2(80,000) + tier3(120,000) = 300,000円
- 特例適用: 300,000 < 330,000 → 330,000円に引き上げ
- 税込: 330,000 + 33,000 = 363,000円

**8,000,000円の計算例（フラグなし）**:
- 通常計算: tier1(100,000) + tier2(80,000) + tier3(120,000) = **300,000円**（240,000円ではない）
- 特例非適用: 300,000円のまま

### テスト期待値

| 売買価格 | 税抜合計 | 消費税 | 税込合計 |
|---|---|---|---|
| 1,000,000円 | 50,000円 | 5,000円 | 55,000円 |
| 2,000,000円 | 100,000円 | 10,000円 | 110,000円 |
| 3,000,000円 | 140,000円 | 14,000円 | 154,000円 |
| 4,000,000円 | 180,000円 | 18,000円 | 198,000円 |
| 5,000,000円 | 210,000円 | 21,000円 | 231,000円 |
| 10,000,000円 | 360,000円 | 36,000円 | 396,000円 |
| 30,000,000円 | 960,000円 | 96,000円 | 1,056,000円 |
| 8,000,000円（低廉特例） | 330,000円 | 33,000円 | 363,000円 |

出典: 国土交通省「宅地建物取引業法の解釈・運用の考え方」第46条関係

---

## 実装上の注意（Phase 1 で判明した仕様）

### 循環依存の回避

`j-law-registry` は `j-law-core` に依存しています。そのため `j-law-core` が `j-law-registry` に依存すると循環依存になりコンパイルできません。

**解決策**: ドメインのパラメータ型（例: `BrokerageFeeParams`）は `j-law-core` 側で定義し、`j-law-registry` はそれをインポートして使います。`j-law-registry` は `j-law-core` の `[dev-dependencies]` にのみ記載し、`[dependencies]` には載せません。

```toml
# crates/j-law-core/Cargo.toml
[dependencies]
thiserror = "1"

[dev-dependencies]
j-law-registry = { path = "../j-law-registry" }  # テスト時のみ
```

```toml
# crates/j-law-registry/Cargo.toml
[dependencies]
j-law-core = { path = "../j-law-core" }  # パラメータ型を参照するため
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

同様に、`j-law-core/src/error.rs` に `serde_json::Error` を含む `ParseError` バリアントを追加してはいけません（`serde_json` は `j-law-core` の依存に含まれていないため）。JSONパースエラーは `j-law-registry` 内で処理し、`RegistryError` の別バリアントとして返してください。

### Integration testのファイル構成

Rustの統合テスト（`tests/`）はルート直下のファイルしか自動検出されません。テストをサブディレクトリで管理する場合は `#[path]` attribute を使ったエントリファイルが必要です。

```
crates/j-law-core/tests/
├── real_estate.rs         ← エントリファイル（必須）
└── real_estate/
    ├── mlitt_examples.rs
    └── edge_cases.rs
```

```rust
// tests/real_estate.rs
#[path = "real_estate/mlitt_examples.rs"]
mod mlitt_examples;

#[path = "real_estate/edge_cases.rs"]
mod edge_cases;
```

エントリファイルなしでサブディレクトリのみを置いても、テストは実行されません。

---

## タスク指示フォーマット

人間からのタスク指示は以下のフォーマットで渡されます。このフォーマット外の指示を受けた場合も、以下の項目が揃っているか確認してから着手してください。

```
## タスク: [タスクID] [タスク名]

### やること
- [具体的な実装内容 1]
- [具体的な実装内容 2]

### 作成/編集するファイル
- crates/j-law-core/src/domains/xxx/calculator.rs（新規作成）

### 完了条件
- `cargo test domains::xxx::` が全グリーン
- floatを一切使用していないこと

### 制約
- f64/f32の使用禁止
- panicを使用禁止（Result型で返す）
```

---

## タスク完了時のセルフレビュー

各タスクの実装が終わったら、コードを提出する前に以下を実行して確認してください。

```sh
# 1. テストが全グリーンか
cargo test --all

# 2. float禁止チェック
grep -r 'f64\|f32' crates/j-law-core/src/ && echo "NG: float使用あり" || echo "OK"

# 3. panic禁止チェック（コア層）
grep -rn 'panic!\|\.unwrap()\|\.expect(' crates/j-law-core/src/ && echo "NG: panic系使用あり" || echo "OK"

# 4. Registry JSONの数値チェック
grep -rn '[0-9]\.[0-9]' crates/j-law-registry/data/ && echo "NG: 小数点あり" || echo "OK"
```

すべてOKになってから完了を報告してください。

---

## エラー対応フロー

| 状況 | 取るべき行動 | 取ってはいけない行動 |
|---|---|---|
| コンパイルエラーが出た | エラー全文を確認し、根本原因を特定してから修正する | 同じ修正を繰り返す |
| テストが失敗した | 期待値と実際の値のどちらが正しいかを公式資料で照合する | テストを削除・`#[ignore]` にする |
| 所有権エラーが収拾つかない | 一旦 `Arc<T>` や `.clone()` で回避し、動作を確認してから最適化する | エラーメッセージを読まずに修正を試みる |
| 計算結果が1円ずれる | `breakdown` の各ステップの中間値を出力し、どのステップで差が出るか特定する | 期待値を書き換えて合わせる |
| 設計と実装が乖離している | このAGENTS.mdの該当セクションを引用して、仕様に合わせて書き直す | 「動いているから問題ない」とみなして放置する |

---

## 実装フェーズとタスク依存関係

各ドメインの実装は以下の順序に従います。

```
Step 0（環境構築・人間が実施）
  └─ Step 1（共通型定義: L1-Types）         ← 全ドメイン共通基盤
  └─ Step 2（共通エラー型: L1-Error）       ← Step 1 と並行可
       └─ Step 3（Registryデータ作成: L2-Data）    ← ドメインごとに実施
            └─ Step 4（Registry Loader: L2-Loader）← ドメインごとに実施
                 └─ Step 5（計算ロジック: L1-Calculator）← ドメインごとに実施
                      └─ Step 6（公式計算例テスト: Integration）← ドメインごとに実施
                           └─ Step 7（Python Binding: L3-PyO3）← 全ドメイン一括
```

Step 1・2（共通基盤）は一度実装すれば全ドメインで使い回します。
新しいドメインを追加する場合はStep 3〜6を繰り返します。

---

## Phase 1（不動産ドメイン）完了チェックリスト

以下が全て満たされた時点でPhase 1完了です。

- [ ] `cargo test --all` が全グリーン
- [ ] `f64`/`f32` が `crates/j-law-core/src/` に存在しない
- [ ] `panic!` が `crates/j-law-core/src/` に存在しない
- [ ] 国交省計算例テストが全件一致（`tests/real_estate/mlitt_examples.rs`）
- [ ] 境界値テストが存在する（200万・400万・800万の各境界）
- [ ] `pip install j-law-core` が動作する
- [ ] Python から `real_estate.calc_brokerage_fee` が呼び出せる
- [ ] 事実認定を要するフラグのdocコメントに警告がある

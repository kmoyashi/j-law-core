# J-Law-Core

日本法令に準拠した計算・検証ライブラリ（Rust / Python / JavaScript / Ruby / Go）

## 概要

J-Law-Core は、日本の法令・告示・省令が定める各種計算を、法的正確性を保証して実装するライブラリです。

- 金額・数値計算に浮動小数点を一切使用せず、整数演算と分数表現で端数処理の再現性を確保
- 法令パラメータ（税率・上限額・経過措置など）をJSONで外部管理し、法改正に対応
- ドメイン単位で法令を追加できる拡張可能なアーキテクチャ
- Rust コアライブラリに加え、Python / JavaScript(WASM) / Ruby / Go の言語バインディングを提供

**実装済みドメイン**

| ドメイン | 対象法令 | 対応告示 |
|---|---|---|
| 不動産（`real_estate`） | 宅地建物取引業法 第46条 | 2019年10月1日施行 / 2024年7月1日施行 |
| 所得税（`income_tax`） | 所得税法 第89条 / 復興財源確保法 第13条 | 2015年1月1日施行 |

---

## 使い方

### Python

```python
from j_law_core.real_estate import calc_brokerage_fee
from j_law_core.income_tax import calc_income_tax

# 媒介報酬の計算（宅建業法 第46条）
result = calc_brokerage_fee(5_000_000, 2024, 8, 1)
print(result.total_with_tax)     # 231000
print(result.total_without_tax)  # 210000
print(result.tax_amount)         # 21000

# 低廉な空き家特例（2024年7月施行・800万円以下）
result = calc_brokerage_fee(8_000_000, 2024, 8, 1, is_low_cost_vacant_house=True)
print(result.total_with_tax)     # 363000

# 所得税の計算（所得税法 第89条）
result = calc_income_tax(5_000_000, 2024, 1, 1, apply_reconstruction_tax=True)
print(result.total_tax)          # 584500
print(result.base_tax)           # 572500
print(result.reconstruction_tax) # 12022
```

### JavaScript (WASM)

```javascript
const { calcBrokerageFee, calcIncomeTax } = require("j-law-wasm");

const fee = calcBrokerageFee(5_000_000, 2024, 8, 1, false);
console.log(fee.totalWithTax); // 231000

const tax = calcIncomeTax(5_000_000, 2024, 1, 1, true);
console.log(tax.totalTax);     // 584500
```

### Ruby

```ruby
require "j_law_core"

result = JLawCore::RealEstate.calc_brokerage_fee(5_000_000, 2024, 8, 1, false)
puts result.total_with_tax  # 231000

result = JLawCore::IncomeTax.calc_income_tax(5_000_000, 2024, 1, 1, true)
puts result.total_tax       # 584500
```

### Go

```go
import jlawcore "github.com/j-law-core/j-law-go"

result, err := jlawcore.CalcBrokerageFee(5_000_000, 2024, 8, 1, false)
fmt.Println(result.TotalWithTax) // 231000

taxResult, err := jlawcore.CalcIncomeTax(5_000_000, 2024, 1, 1, true)
fmt.Println(taxResult.TotalTax)  // 584500
```

### Rust

```rust
use j_law_core::domains::real_estate::{
    calculator::calculate_brokerage_fee,
    context::RealEstateContext,
    policy::StandardMliitPolicy,
};
use j_law_registry::load_brokerage_fee_params;
use std::collections::HashSet;

let ctx = RealEstateContext {
    price: 5_000_000,
    target_date: (2024, 8, 1),
    flags: HashSet::new(),
    policy: Box::new(StandardMliitPolicy),
};
let params = load_brokerage_fee_params(ctx.target_date)?;
let result = calculate_brokerage_fee(&ctx, &params)?;
println!("税込報酬額: {}円", result.total_with_tax.as_yen()); // 231000
```

---

## プロジェクト構成

```
j-law-core/
├── crates/
│   ├── j-law-core/               # コアライブラリ（型・エラー・計算ロジック）
│   │   └── src/
│   │       ├── types/            # FinalAmount, Rate, RoundingStrategy
│   │       ├── error.rs          # JLawError 階層
│   │       └── domains/
│   │           ├── real_estate/  # 不動産ドメイン（宅建業法 第46条）
│   │           └── income_tax/   # 所得税ドメイン（所得税法 第89条）
│   ├── j-law-registry/           # 法令パラメータ管理（JSON）
│   │   └── data/
│   │       ├── real_estate/      # 宅建業法告示パラメータ
│   │       └── income_tax/       # 所得税法パラメータ
│   ├── j-law-python/             # Python バインディング（PyO3）
│   ├── j-law-wasm/               # WASM/JavaScript バインディング（wasm-bindgen）
│   ├── j-law-ruby/               # Ruby バインディング（Magnus）
│   ├── j-law-cgo/                # C FFI（Go 向け staticlib）
│   └── j-law-go/                 # Go バインディング（CGo）
├── tests/
│   └── fixtures/                 # 全言語共通テストフィクスチャ（JSON）
│       ├── real_estate.json
│       └── income_tax.json
├── Dockerfile                    # マルチステージテスト環境
└── docker-compose.yml            # 全言語テスト一括実行
```

---

## 計算仕様

### 不動産ドメイン — 媒介報酬（宅建業法 第46条）

**3段階ティア計算**

| 売買価格の範囲 | 率 |
|---|---|
| 200万円以下の部分 | 5% |
| 200万円超〜400万円以下の部分 | 4% |
| 400万円超の部分 | 3% |

端数処理: 各ティアで切り捨て → 合計 → 消費税10%（切り捨て）

**低廉な空き家特例（2024年7月1日〜）**: 売買価格が800万円以下で `IsLowCostVacantHouse` フラグを指定した場合、税抜報酬額が330,000円に引き上げられます。

> **注意**: `IsLowCostVacantHouse` フラグの事実認定はこのライブラリの責任範囲外です。

**計算例**

| 売買価格 | 税抜合計 | 消費税 | 税込合計 |
|---|---|---|---|
| 1,000,000円 | 50,000円 | 5,000円 | 55,000円 |
| 5,000,000円 | 210,000円 | 21,000円 | 231,000円 |
| 10,000,000円 | 360,000円 | 36,000円 | 396,000円 |
| 30,000,000円 | 960,000円 | 96,000円 | 1,056,000円 |
| 8,000,000円（低廉特例） | 330,000円 | 33,000円 | 363,000円 |

### 所得税ドメイン — 所得税額（所得税法 第89条）

**速算表方式（7段階累進課税）**

| 課税所得金額 | 税率 | 控除額 |
|---|---|---|
| 〜195万円 | 5% | 0円 |
| 195万円超〜330万円 | 10% | 97,500円 |
| 330万円超〜695万円 | 20% | 427,500円 |
| 695万円超〜900万円 | 23% | 636,000円 |
| 900万円超〜1,800万円 | 33% | 1,536,000円 |
| 1,800万円超〜4,000万円 | 40% | 2,796,000円 |
| 4,000万円超 | 45% | 4,796,000円 |

- **復興特別所得税**: 基準所得税額 × 2.1%（2013〜2037年）
- **申告納税額**: 100円未満切り捨て

---

## ビルド・テスト

### Docker（推奨 — 全言語一括テスト）

```sh
# 全言語テスト一括実行
docker compose up test-all --build

# 個別言語テスト
docker compose up test-rust --build
docker compose up test-python --build
docker compose up test-wasm --build
docker compose up test-ruby --build
docker compose up test-go --build
```

### ローカル

```sh
# Rust コアテスト
cargo test --all

# Python
pip install maturin pytest
maturin develop -m crates/j-law-python/Cargo.toml
pytest crates/j-law-python/tests/ -v

# WASM/JS
wasm-pack build --target nodejs crates/j-law-wasm
node --test crates/j-law-wasm/tests/*.test.mjs

# Ruby
cd crates/j-law-ruby && bundle install && bundle exec rake test

# Go
cd crates/j-law-go && make test
```

### テストフィクスチャ

全言語のテストは `tests/fixtures/` 配下の共通 JSON フィクスチャからテストケースを読み込むデータ駆動テスト方式です。テストケースの追加・修正は JSON を編集するだけで全言語に反映されます。

---

## コントリビューション

Issue・Pull Request を歓迎します。コードを変更・追加する場合は以下の点に従ってください。

- `crates/j-law-core/src/` に `f64`/`f32` を使わないこと（`clippy.toml` で禁止設定）
- `crates/j-law-core/src/` に `panic!`/`unwrap()`/`expect()` を使わないこと（`clippy.toml` で禁止設定）
- 公開APIにはRustdocで根拠条文を明記すること
- 変更に対応するテストを必ず追加すること
- 新規ドメインを追加する場合は既存ドメイン（`real_estate`, `income_tax`）の構成に倣うこと
- コミット前に `cargo clippy --all-targets --all-features -- -D warnings` が通ることを確認すること

詳細なコーディングルールは [AGENTS.md](AGENTS.md) を参照してください。

---

## 免責事項

本ライブラリは法的助言を提供するものではありません。計算結果は参考情報であり、実際の手続きにおいては必ず有資格者または弁護士に確認してください。法改正により計算ロジックが変わる場合があります。

---

## ライセンス

[MIT License](LICENSE)

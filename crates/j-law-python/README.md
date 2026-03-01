# j-law-core (Python)

日本の法令に基づく各種計算を提供する Python バインディング。

Rust コアライブラリ（j-law-core）を [PyO3](https://pyo3.rs/) 経由でラップしています。
浮動小数点演算を一切使用せず、整数演算で端数処理の再現性を保証します。

> [!WARNING]
> **アルファ版・AI生成コードに関する注意事項**
>
> - 本ライブラリは現在 **アルファ版（v0.0.1）** です。API は予告なく変更される場合があります。
> - コードの大部分は **AI（LLM）によって生成** されており、人間による網羅的なレビューが十分に行われていません。
> - 計算結果を実際の法的手続きや業務判断に用いる際は、必ず有資格者または専門家に確認してください。

## インストール

```sh
pip install j-law-core
```

ソースからビルドする場合:

```sh
pip install maturin
maturin develop -m crates/j-law-python/Cargo.toml
```

## 使い方

### 不動産ドメイン — 媒介報酬（宅建業法 第46条）

```python
from j_law_core.real_estate import calc_brokerage_fee

# 売買価格 500万円、2024年8月1日基準
result = calc_brokerage_fee(5_000_000, 2024, 8, 1)

print(result.total_with_tax)           # 231000（税込）
print(result.total_without_tax)        # 210000（税抜）
print(result.tax_amount)               # 21000（消費税）
print(result.low_cost_special_applied) # False

# 各ティアの内訳
for step in result.breakdown:
    print(step.label, step.base_amount, step.result)
    # tier1  2000000  100000
    # tier2  2000000  80000
    # tier3  1000000  30000

# 低廉な空き家特例（2024年7月1日施行・800万円以下）
# WARNING: 対象物件が特例に該当するかの事実認定は呼び出し元の責任
result = calc_brokerage_fee(8_000_000, 2024, 8, 1, is_low_cost_vacant_house=True)
print(result.total_with_tax)           # 363000
print(result.low_cost_special_applied) # True
```

### 所得税ドメイン — 所得税額（所得税法 第89条）

```python
from j_law_core.income_tax import calc_income_tax

# 課税所得 500万円（1,000円未満切り捨て済みの値を渡すこと）
result = calc_income_tax(5_000_000, 2024, 1, 1)

print(result.total_tax)                   # 584500（申告納税額・100円未満切り捨て）
print(result.base_tax)                    # 572500（基準所得税額）
print(result.reconstruction_tax)          # 12022（復興特別所得税）
print(result.reconstruction_tax_applied)  # True

# 復興特別所得税を適用しない場合
result = calc_income_tax(5_000_000, 2024, 1, 1, apply_reconstruction_tax=False)
print(result.total_tax)                   # 572500
```

### 印紙税ドメイン — 印紙税額（印紙税法 別表第一）

```python
from j_law_core.stamp_tax import calc_stamp_tax

# 契約金額 500万円（不動産譲渡契約書）
result = calc_stamp_tax(5_000_000, 2024, 8, 1)

print(result.tax_amount)            # 2000（印紙税額）
print(result.bracket_label)         # 適用ブラケット名
print(result.reduced_rate_applied)  # False

# 軽減税率適用（租税特別措置法 第91条）
# WARNING: 対象文書が軽減措置の適用要件を満たすかの事実認定は呼び出し元の責任
result = calc_stamp_tax(5_000_000, 2024, 8, 1, is_reduced_rate_applicable=True)
print(result.reduced_rate_applied)  # True
```

## API リファレンス

### `j_law_core.real_estate`

#### `calc_brokerage_fee(price, year, month, day, is_low_cost_vacant_house=False)`

宅建業法第46条に基づく媒介報酬を計算する。

| 引数 | 型 | 説明 |
|---|---|---|
| `price` | `int` | 売買価格（円） |
| `year` | `int` | 基準日（年） |
| `month` | `int` | 基準日（月） |
| `day` | `int` | 基準日（日） |
| `is_low_cost_vacant_house` | `bool` | 低廉な空き家特例フラグ（デフォルト: `False`） |

**戻り値: `BrokerageFeeResult`**

| プロパティ | 型 | 説明 |
|---|---|---|
| `total_without_tax` | `int` | 税抜合計額（円） |
| `total_with_tax` | `int` | 税込合計額（円） |
| `tax_amount` | `int` | 消費税額（円） |
| `low_cost_special_applied` | `bool` | 低廉な空き家特例が適用されたか |
| `breakdown` | `list[BreakdownStep]` | 各ティアの計算内訳 |

**例外: `ValueError`** — 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合。

---

### `j_law_core.income_tax`

#### `calc_income_tax(taxable_income, year, month, day, apply_reconstruction_tax=True)`

所得税法第89条に基づく所得税額を計算する。

| 引数 | 型 | 説明 |
|---|---|---|
| `taxable_income` | `int` | 課税所得金額（円・1,000円未満切り捨て済み） |
| `year` | `int` | 対象年度（年） |
| `month` | `int` | 基準日（月） |
| `day` | `int` | 基準日（日） |
| `apply_reconstruction_tax` | `bool` | 復興特別所得税を適用するか（デフォルト: `True`） |

**戻り値: `IncomeTaxResult`**

| プロパティ | 型 | 説明 |
|---|---|---|
| `base_tax` | `int` | 基準所得税額（円） |
| `reconstruction_tax` | `int` | 復興特別所得税額（円） |
| `total_tax` | `int` | 申告納税額（円・100円未満切り捨て） |
| `reconstruction_tax_applied` | `bool` | 復興特別所得税が適用されたか |
| `breakdown` | `list[IncomeTaxStep]` | 速算表の計算内訳 |

**例外: `ValueError`** — 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合。

---

### `j_law_core.stamp_tax`

#### `calc_stamp_tax(contract_amount, year, month, day, is_reduced_rate_applicable=False)`

印紙税法 別表第一（第1号文書）に基づく印紙税額を計算する。

| 引数 | 型 | 説明 |
|---|---|---|
| `contract_amount` | `int` | 契約金額（円） |
| `year` | `int` | 契約書作成日（年） |
| `month` | `int` | 契約書作成日（月） |
| `day` | `int` | 契約書作成日（日） |
| `is_reduced_rate_applicable` | `bool` | 軽減税率適用フラグ（デフォルト: `False`） |

**戻り値: `StampTaxResult`**

| プロパティ | 型 | 説明 |
|---|---|---|
| `tax_amount` | `int` | 印紙税額（円） |
| `bracket_label` | `str` | 適用されたブラケットの表示名 |
| `reduced_rate_applied` | `bool` | 軽減税率が適用されたか |

**例外: `ValueError`** — 契約金額が不正、または対象日に有効な法令パラメータが存在しない場合。

## テスト

```sh
pip install pytest
pytest crates/j-law-python/tests/ -v
```

## ライセンス

[MIT License](../../LICENSE)

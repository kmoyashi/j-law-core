# j-law-core (Python)

日本の法令に基づく各種計算を提供する Python バインディングです。

内部では `j-law-c-ffi` の C ABI を `ctypes` 経由で呼び出し、Rust コアの整数演算ロジックを Python から利用します。

> [!WARNING]
> **アルファ版・AI生成コードに関する注意事項**
>
> - 本ライブラリは現在 **アルファ版（v0.0.1）** です。API は予告なく変更される場合があります。
> - コードの大部分は **AI（LLM）によって生成** されており、人間による網羅的なレビューが十分に行われていません。
> - 計算結果を実際の法的手続きや業務判断に用いる際は、必ず有資格者または専門家に確認してください。

## インストール

wheel を利用する場合:

```sh
pip install j-law-python
```

ソースからビルドする場合:

```sh
pip install ./crates/j-law-python
```

ローカル checkout を直接 import する場合は、先に `j-law-c-ffi` をビルドしてください。

```sh
cargo build -p j-law-c-ffi
```

共有ライブラリの探索先を明示したい場合は `JLAW_PYTHON_C_FFI_LIB` を指定できます。

## 使い方

### 不動産ドメイン — 媒介報酬（宅建業法 第46条）

```python
import datetime

from j_law_python.real_estate import calc_brokerage_fee

result = calc_brokerage_fee(5_000_000, datetime.date(2024, 8, 1))

print(result.total_with_tax)           # 231000
print(result.total_without_tax)        # 210000
print(result.tax_amount)               # 21000
print(result.low_cost_special_applied) # False

for step in result.breakdown:
    print(step.label, step.base_amount, step.result)
    # tier1 2000000 100000
    # tier2 2000000 80000
    # tier3 1000000 30000
```

### 所得税ドメイン — 所得税額（所得税法 第89条）

```python
import datetime

from j_law_python.income_tax import calc_income_tax

result = calc_income_tax(5_000_000, datetime.date(2024, 1, 1))

print(result.total_tax)                   # 584500
print(result.base_tax)                    # 572500
print(result.reconstruction_tax)          # 12022
print(result.reconstruction_tax_applied)  # True
```

### 消費税ドメイン — 消費税額（消費税法 第29条）

```python
import datetime

from j_law_python.consumption_tax import calc_consumption_tax

result = calc_consumption_tax(100_000, datetime.date(2024, 1, 1))

print(result.tax_amount)          # 10000
print(result.amount_with_tax)     # 110000
print(result.is_reduced_rate)     # False
```

### 印紙税ドメイン — 印紙税額（印紙税法 別表第一）

```python
import datetime

from j_law_python.stamp_tax import calc_stamp_tax

result = calc_stamp_tax(5_000_000, datetime.date(2024, 8, 1))

print(result.tax_amount)            # 2000
print(result.bracket_label)         # 適用ブラケット名
print(result.reduced_rate_applied)  # False

construction = calc_stamp_tax(
    1_500_000,
    datetime.date(2024, 8, 1),
    is_reduced_rate_applicable=True,
    document_kind="construction_contract",
)

print(construction.tax_amount)      # 200
```

## API

- `j_law_python.real_estate.calc_brokerage_fee(price, date, is_low_cost_vacant_house=False, is_seller=False)`
- `j_law_python.income_tax.calc_income_tax(taxable_income, date, apply_reconstruction_tax=True)`
- `j_law_python.consumption_tax.calc_consumption_tax(amount, date, is_reduced_rate=False)`
- `j_law_python.stamp_tax.calc_stamp_tax(contract_amount, date, is_reduced_rate_applicable=False, document_kind="real_estate_transfer")`

すべての API は `datetime.date` を受け取り、型不一致は `TypeError`、入力不正や法令適用外日付は `ValueError` を送出します。

## テスト

```sh
pip install pytest
pytest crates/j-law-python/tests/ -v
```

## ライセンス

[MIT License](../../LICENSE)

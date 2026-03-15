# j-law-python

日本の法令に基づく各種計算を提供する Python バインディングです。

内部では `j-law-c-ffi` の C ABI を `ctypes` 経由で呼び出し、Rust コアの整数演算ロジックを Python から利用します。

> [!WARNING]
> **アルファ版・AI生成コードに関する注意事項**
>
> - 本ライブラリは現在 **アルファ版（v0.0.1）** です。API は予告なく変更される場合があります。
> - コードの大部分は **AI（LLM）によって生成** されており、人間による網羅的なレビューが十分に行われていません。
> - 計算結果を実際の法的手続きや業務判断に用いる際は、必ず有資格者または専門家に確認してください。

## 対応機能

- `j_law_python.consumption_tax.calc_consumption_tax`
- `j_law_python.real_estate.calc_brokerage_fee`
- `j_law_python.income_tax.calc_income_tax`
- `j_law_python.income_tax.calc_income_deductions`
- `j_law_python.income_tax.calc_income_tax_assessment`
- `j_law_python.stamp_tax.calc_stamp_tax`
- `j_law_python.withholding_tax.calc_withholding_tax`

## インストール

```sh
pip install j-law-python
```

公開サポート範囲は次のとおりです。

- CPython `3.10` から `3.14`
- PyPI wheel: `linux/x86_64` `linux/aarch64` `macos/x86_64` `macos/arm64` `windows/amd64`
- それ以外の環境は source build 扱いです。Rust `1.94.0` が必要です。

リポジトリ checkout を直接 install / import する場合は、先に `j-law-c-ffi` をビルドしてください。

```sh
cargo build -p j-law-c-ffi
pip install ./crates/j-law-python
```

共有ライブラリの探索先を明示したい場合は `JLAW_PYTHON_C_FFI_LIB` を指定できます。

## クイックスタート

```python
import datetime

from j_law_python.consumption_tax import calc_consumption_tax
from j_law_python.income_tax import IncomeDeductionInput, calc_income_tax_assessment
from j_law_python.real_estate import calc_brokerage_fee
from j_law_python.stamp_tax import calc_stamp_tax
from j_law_python.withholding_tax import (
    WithholdingTaxCategory,
    calc_withholding_tax,
)

print(calc_consumption_tax(100_000, datetime.date(2024, 1, 1)).tax_amount)
print(calc_brokerage_fee(5_000_000, datetime.date(2024, 8, 1)).total_with_tax)

assessment = calc_income_tax_assessment(
    IncomeDeductionInput(
        total_income_amount=8_000_000,
        date=datetime.date(2024, 1, 1),
        social_insurance_premium_paid=600_000,
    )
)
print(assessment.tax.total_tax)

print(
    calc_stamp_tax(
        "article1_real_estate_transfer",
        5_000_000,
        datetime.date(2024, 8, 1),
    ).tax_amount
)

print(
    calc_withholding_tax(
        1_500_000,
        datetime.date(2026, 1, 1),
        WithholdingTaxCategory.PROFESSIONAL_FEE,
    ).tax_amount
)
```

## API メモ

- すべての金額は整数円です。
- すべての API は `datetime.date` を受け取り、型不一致は `TypeError` を送出します。
- 入力不正や法令適用外日付は `ValueError` を送出します。
- `calc_stamp_tax()` の `document_code` / `flags` は文字列で指定します。
- `calc_withholding_tax()` の `category` は enum / 文字列 / 整数を受け取れます。

## テスト

```sh
pip install pytest
pytest crates/j-law-python/tests/ -v
```

## 関連ドキュメント

- [リポジトリ README](../../README.md)
- [利用ガイド](../../docs/usage.md)

## ライセンス

[MIT License](../../LICENSE)

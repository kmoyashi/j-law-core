# j-law-python

日本法令計算の PoC を Python から試すためのバインディングです。

内部では `j-law-c-ffi` の C ABI を `ctypes` 経由で呼び出し、Rust コアの整数演算ロジックを Python から利用します。

> [!WARNING]
> **PoC / アルファ版に関する注意事項**
>
> - 本ライブラリは現在 **`v0.0.1` のアルファ版**です。API と配布形態は予告なく変更される場合があります。
> - この binding が返す計算結果について、法的正確性、完全性、最新性、個別事案への適合性は保証しません。
> - コードの一部には **AI 生成 / AI 補助**による実装が含まれ、人手による全面レビューは完了していません。
> - 税務申告や契約実務の唯一の根拠として使用せず、一次資料と専門家で検証してください。
> - 詳細は [プロジェクトステータスと免責](../../docs/project-status.md) を参照してください。

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

CI と publish workflow で検証している組み合わせは次のとおりです。

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

- [プロジェクトステータスと免責](../../docs/project-status.md)
- [リポジトリ README](../../README.md)
- [利用ガイド](../../docs/usage.md)

## ライセンス

[MIT License](../../LICENSE)

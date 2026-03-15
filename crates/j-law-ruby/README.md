# j_law_ruby

日本の法令に基づく各種計算を提供する Ruby バインディングです。

内部では `j-law-c-ffi` の C ABI を `ffi` gem 経由で呼び出し、Rust コアの整数演算ロジックを Ruby から利用します。

## 対応機能

- `JLawRuby::ConsumptionTax.calc_consumption_tax`
- `JLawRuby::RealEstate.calc_brokerage_fee`
- `JLawRuby::IncomeTax.calc_income_tax`
- `JLawRuby::IncomeTax.calc_income_deductions`
- `JLawRuby::IncomeTax.calc_income_tax_assessment`
- `JLawRuby::StampTax.calc_stamp_tax`
- `JLawRuby::WithholdingTax.calc_withholding_tax`

## インストール

source gem として配布する前提です。`gem install` 時に Rust 1.94.0 toolchain が必要です。

```sh
gem install j_law_ruby
```

開発環境では次を実行します。

```sh
cd crates/j-law-ruby
bundle install
bundle exec rake compile
bundle exec rake test
```

## クイックスタート

```ruby
require "date"
require "j_law_ruby"

date = Date.new(2024, 8, 1)

puts JLawRuby::ConsumptionTax.calc_consumption_tax(100_000, Date.new(2024, 1, 1), false).tax_amount
puts JLawRuby::RealEstate.calc_brokerage_fee(5_000_000, date, false, false).total_with_tax

assessment = JLawRuby::IncomeTax.calc_income_tax_assessment(
  8_000_000,
  Date.new(2024, 1, 1),
  social_insurance_premium_paid: 600_000
)
puts assessment.tax.total_tax

puts JLawRuby::StampTax.calc_stamp_tax(:article1_real_estate_transfer, 5_000_000, date).tax_amount

puts JLawRuby::WithholdingTax.calc_withholding_tax(
  1_500_000,
  Date.new(2026, 1, 1),
  :professional_fee
).tax_amount
```

## API メモ

- すべての金額は整数円です。
- すべての API は `Date` / `DateTime` を受け取り、型不一致は `TypeError` を送出します。
- 法令適用外日付や入力不正は `RuntimeError` を送出します。
- `calc_stamp_tax()` の `document_code` / `flags` は `Symbol` または `String` を受け取ります。
- `calc_withholding_tax()` の `category` は `:manuscript_and_lecture` / `:professional_fee` / `:exclusive_contract_fee` などを指定します。

## Gem ビルド

```sh
cd crates/j-law-ruby
bundle install
bundle exec rake build
```

`rake build` は source gem 用に必要な Rust ソースを `vendor/rust/` へ同期してから gem を生成します。

## 関連ドキュメント

- [リポジトリ README](../../README.md)
- [利用ガイド](../../docs/usage.md)

# j_law_ruby

日本の法令に基づく各種計算を提供する Ruby バインディングです。

内部では `j-law-cgo` の C ABI を `ffi` gem 経由で呼び出し、Rust コアの整数演算ロジックを Ruby から利用します。

## インストール

source gem として配布する前提です。`gem install` 時に Rust toolchain が必要です。

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

## 使い方

```ruby
require "date"
require "j_law_ruby"
```

### 消費税

```ruby
result = JLawRuby::ConsumptionTax.calc_consumption_tax(
  100_000,
  Date.new(2024, 1, 1),
  false
)

puts result.tax_amount         # 10000
puts result.amount_with_tax    # 110000
puts result.is_reduced_rate?   # false
```

### 不動産: 媒介報酬

```ruby
result = JLawRuby::RealEstate.calc_brokerage_fee(
  5_000_000,
  Date.new(2024, 8, 1),
  false,
  false
)

puts result.total_without_tax        # 210000
puts result.total_with_tax           # 231000
puts result.low_cost_special_applied? # false
puts result.breakdown.map { |step| step[:label] } # ["tier1", "tier2", "tier3"]
```

### 所得税

```ruby
result = JLawRuby::IncomeTax.calc_income_tax(
  5_000_000,
  Date.new(2024, 1, 1),
  true
)

puts result.base_tax                     # 572500
puts result.reconstruction_tax           # 12022
puts result.total_tax                    # 584500
puts result.reconstruction_tax_applied?  # true
```

### 印紙税

```ruby
result = JLawRuby::StampTax.calc_stamp_tax(
  5_000_000,
  Date.new(2024, 8, 1),
  true
)

puts result.tax_amount             # 1000
puts result.bracket_label          # "500万円以下"
puts result.reduced_rate_applied?  # true
```

## API

- `JLawRuby::ConsumptionTax.calc_consumption_tax(amount, date, is_reduced_rate = false)`
- `JLawRuby::RealEstate.calc_brokerage_fee(price, date, is_low_cost_vacant_house, is_seller)`
- `JLawRuby::IncomeTax.calc_income_tax(taxable_income, date, apply_reconstruction_tax)`
- `JLawRuby::StampTax.calc_stamp_tax(contract_amount, date, is_reduced_rate_applicable)`

すべての API は `Date` または `DateTime` を受け取り、日付型以外が渡された場合は `TypeError` を送出します。法令パラメータの適用外日付や入力不正は `RuntimeError` を送出します。

## Gem ビルド

```sh
cd crates/j-law-ruby
bundle install
bundle exec rake build
```

`rake build` は source gem 用に必要な Rust ソースを `vendor/rust/` へ同期してから gem を生成します。

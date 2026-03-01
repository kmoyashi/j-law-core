# j_law_ruby

日本の法令に基づく各種計算を提供する Ruby バインディング（gem）。

Rust コアライブラリ（j-law-core）を [Magnus](https://github.com/matsadler/magnus) 経由でラップしています。
浮動小数点演算を一切使用せず、整数演算で端数処理の再現性を保証します。

> [!WARNING]
> **アルファ版・AI生成コードに関する注意事項**
>
> - 本ライブラリは現在 **アルファ版（v0.0.1）** です。API は予告なく変更される場合があります。
> - コードの大部分は **AI（LLM）によって生成** されており、人間による網羅的なレビューが十分に行われていません。
> - 計算結果を実際の法的手続きや業務判断に用いる際は、必ず有資格者または専門家に確認してください。

## インストール

```sh
gem install j_law_ruby
```

`Gemfile` に追加する場合:

```ruby
gem "j_law_ruby"
```

ソースからビルドする場合:

```sh
cd crates/j-law-ruby
bundle install
bundle exec rake build
```

## 使い方

```ruby
require "j_law_ruby"
```

### 不動産ドメイン — 媒介報酬（宅建業法 第46条）

```ruby
# 売買価格 500万円、2024年8月1日基準
result = JLawRuby::RealEstate.calc_brokerage_fee(5_000_000, 2024, 8, 1, false)

puts result.total_with_tax           # 231000（税込）
puts result.total_without_tax        # 210000（税抜）
puts result.tax_amount               # 21000（消費税）
puts result.low_cost_special_applied? # false

# 各ティアの内訳
result.breakdown.each do |step|
  puts "#{step[:label]}: #{step[:base_amount]}円 × #{step[:rate_numer]}/#{step[:rate_denom]} = #{step[:result]}円"
  # tier1: 2000000円 × 5/100 = 100000円
  # tier2: 2000000円 × 4/100 = 80000円
  # tier3: 1000000円 × 3/100 = 30000円
end

# 低廉な空き家特例（2024年7月1日施行・800万円以下）
# WARNING: 対象物件が特例に該当するかの事実認定は呼び出し元の責任
result = JLawRuby::RealEstate.calc_brokerage_fee(8_000_000, 2024, 8, 1, true)
puts result.total_with_tax            # 363000
puts result.low_cost_special_applied? # true
```

### 所得税ドメイン — 所得税額（所得税法 第89条）

```ruby
# 課税所得 500万円（1,000円未満切り捨て済みの値を渡すこと）
result = JLawRuby::IncomeTax.calc_income_tax(5_000_000, 2024, 1, 1, true)

puts result.total_tax                    # 584500（申告納税額・100円未満切り捨て）
puts result.base_tax                     # 572500（基準所得税額）
puts result.reconstruction_tax          # 12022（復興特別所得税）
puts result.reconstruction_tax_applied? # true

# 復興特別所得税を適用しない場合
result = JLawRuby::IncomeTax.calc_income_tax(5_000_000, 2024, 1, 1, false)
puts result.total_tax                    # 572500
```

### 印紙税ドメイン — 印紙税額（印紙税法 別表第一）

```ruby
# 契約金額 500万円（不動産譲渡契約書）
result = JLawRuby::StampTax.calc_stamp_tax(5_000_000, 2024, 8, 1, false)

puts result.tax_amount            # 2000（印紙税額）
puts result.bracket_label         # 適用ブラケット名
puts result.reduced_rate_applied? # false

# 軽減税率適用（租税特別措置法 第91条）
# WARNING: 対象文書が軽減措置の適用要件を満たすかの事実認定は呼び出し元の責任
result = JLawRuby::StampTax.calc_stamp_tax(5_000_000, 2024, 8, 1, true)
puts result.reduced_rate_applied? # true
```

## API リファレンス

### `JLawRuby::RealEstate`

#### `.calc_brokerage_fee(price, year, month, day, is_low_cost_vacant_house)`

宅建業法第46条に基づく媒介報酬を計算する。

| 引数 | 型 | 説明 |
|---|---|---|
| `price` | `Integer` | 売買価格（円） |
| `year` | `Integer` | 基準日（年） |
| `month` | `Integer` | 基準日（月） |
| `day` | `Integer` | 基準日（日） |
| `is_low_cost_vacant_house` | `true`/`false` | 低廉な空き家特例フラグ |

**戻り値: `JLawRuby::RealEstate::BrokerageFeeResult`**

| メソッド | 戻り値型 | 説明 |
|---|---|---|
| `total_without_tax` | `Integer` | 税抜合計額（円） |
| `total_with_tax` | `Integer` | 税込合計額（円） |
| `tax_amount` | `Integer` | 消費税額（円） |
| `low_cost_special_applied?` | `true`/`false` | 低廉な空き家特例が適用されたか |
| `breakdown` | `Array<Hash>` | 各ティアの計算内訳（キー: `:label`, `:base_amount`, `:rate_numer`, `:rate_denom`, `:result`） |

**例外: `RuntimeError`** — 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合。

---

### `JLawRuby::IncomeTax`

#### `.calc_income_tax(taxable_income, year, month, day, apply_reconstruction_tax)`

所得税法第89条に基づく所得税額を計算する。

| 引数 | 型 | 説明 |
|---|---|---|
| `taxable_income` | `Integer` | 課税所得金額（円・1,000円未満切り捨て済み） |
| `year` | `Integer` | 対象年度（年） |
| `month` | `Integer` | 基準日（月） |
| `day` | `Integer` | 基準日（日） |
| `apply_reconstruction_tax` | `true`/`false` | 復興特別所得税を適用するか |

**戻り値: `JLawRuby::IncomeTax::IncomeTaxResult`**

| メソッド | 戻り値型 | 説明 |
|---|---|---|
| `base_tax` | `Integer` | 基準所得税額（円） |
| `reconstruction_tax` | `Integer` | 復興特別所得税額（円） |
| `total_tax` | `Integer` | 申告納税額（円・100円未満切り捨て） |
| `reconstruction_tax_applied?` | `true`/`false` | 復興特別所得税が適用されたか |
| `breakdown` | `Array<Hash>` | 速算表の計算内訳（キー: `:label`, `:taxable_income`, `:rate_numer`, `:rate_denom`, `:deduction`, `:result`） |

**例外: `RuntimeError`** — 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合。

---

### `JLawRuby::StampTax`

#### `.calc_stamp_tax(contract_amount, year, month, day, is_reduced_rate_applicable)`

印紙税法 別表第一（第1号文書）に基づく印紙税額を計算する。

| 引数 | 型 | 説明 |
|---|---|---|
| `contract_amount` | `Integer` | 契約金額（円） |
| `year` | `Integer` | 契約書作成日（年） |
| `month` | `Integer` | 契約書作成日（月） |
| `day` | `Integer` | 契約書作成日（日） |
| `is_reduced_rate_applicable` | `true`/`false` | 軽減税率適用フラグ |

**戻り値: `JLawRuby::StampTax::StampTaxResult`**

| メソッド | 戻り値型 | 説明 |
|---|---|---|
| `tax_amount` | `Integer` | 印紙税額（円） |
| `bracket_label` | `String` | 適用されたブラケットの表示名 |
| `reduced_rate_applied?` | `true`/`false` | 軽減税率が適用されたか |

**例外: `RuntimeError`** — 契約金額が不正、または対象日に有効な法令パラメータが存在しない場合。

## テスト

```sh
cd crates/j-law-ruby
bundle install
bundle exec rake test
```

## ライセンス

[MIT License](../../LICENSE)

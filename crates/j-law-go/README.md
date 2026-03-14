# j-law-go

日本の法令に基づく各種計算を提供する Go バインディング。

`j-law-c-ffi` が提供する C ABI を CGo 経由で利用しています。
浮動小数点演算を一切使用せず、整数演算で端数処理の再現性を保証します。

> [!WARNING]
> **アルファ版・AI生成コードに関する注意事項**
>
> - 本ライブラリは現在 **アルファ版（v0.0.1）** です。API は予告なく変更される場合があります。
> - コードの大部分は **AI（LLM）によって生成** されており、人間による網羅的なレビューが十分に行われていません。
> - 計算結果を実際の法的手続きや業務判断に用いる際は、必ず有資格者または専門家に確認してください。

> [!NOTE]
> **CGo 静的リンクについて**
>
> このパッケージは `j-law-c-ffi` の C ABI を CGo 経由で利用します。
> `go get` / `go test` の前に必ず `make build-rust` を実行してください。

## インストール

```sh
go get github.com/kmoyashi/j-law-go
```

使用前に Rust staticlib をビルドします（リポジトリのルートが必要）:

```sh
cd crates/j-law-go
make build-rust   # target/debug/libj_law_c_ffi.a を生成
```

## 使い方

```go
import jlawcore "github.com/kmoyashi/j-law-go"
```

### 不動産ドメイン — 媒介報酬（宅建業法 第46条）

```go
// 売買価格 500万円、2024年8月1日基準
result, err := jlawcore.CalcBrokerageFee(5_000_000, 2024, 8, 1, false, false)
if err != nil {
    log.Fatal(err)
}

fmt.Println(result.TotalWithTax)           // 231000（税込）
fmt.Println(result.TotalWithoutTax)        // 210000（税抜）
fmt.Println(result.TaxAmount)              // 21000（消費税）
fmt.Println(result.LowCostSpecialApplied)  // false

// 各ティアの内訳
for _, step := range result.Breakdown {
    fmt.Printf("%s: %d円 × %d/%d = %d円\n",
        step.Label, step.BaseAmount, step.RateNumer, step.RateDenom, step.Result)
    // tier1: 2000000円 × 5/100 = 100000円
    // tier2: 2000000円 × 4/100 = 80000円
    // tier3: 1000000円 × 3/100 = 30000円
}

// 低廉な空き家特例（2024年7月1日施行・800万円以下・売主買主双方）
// WARNING: 対象物件が特例に該当するかの事実認定は呼び出し元の責任
special, err := jlawcore.CalcBrokerageFee(8_000_000, 2024, 8, 1, true, false)
if err != nil {
    log.Fatal(err)
}
fmt.Println(special.TotalWithTax)          // 363000
fmt.Println(special.LowCostSpecialApplied) // true

// 低廉な空き家特例（2018年1月〜2024年6月・400万円以下・売主のみ）
special2018, err := jlawcore.CalcBrokerageFee(4_000_000, 2022, 4, 1, true, true)
if err != nil {
    log.Fatal(err)
}
fmt.Println(special2018.TotalWithTax)          // 198000
fmt.Println(special2018.LowCostSpecialApplied) // true
```

### 所得税ドメイン — 所得税額（所得税法 第89条）

```go
// 課税所得 500万円（1,000円未満切り捨て済みの値を渡すこと）
result, err := jlawcore.CalcIncomeTax(5_000_000, 2024, 1, 1, true)
if err != nil {
    log.Fatal(err)
}

fmt.Println(result.TotalTax)                  // 584500（申告納税額・100円未満切り捨て）
fmt.Println(result.BaseTax)                   // 572500（基準所得税額）
fmt.Println(result.ReconstructionTax)         // 12022（復興特別所得税）
fmt.Println(result.ReconstructionTaxApplied)  // true

// 復興特別所得税を適用しない場合
result2, err := jlawcore.CalcIncomeTax(5_000_000, 2024, 1, 1, false)
if err != nil {
    log.Fatal(err)
}
fmt.Println(result2.TotalTax)                 // 572500
```

### 印紙税ドメイン — 印紙税額（印紙税法 別表第一）

```go
date := time.Date(2024, time.August, 1, 0, 0, 0, 0, time.UTC)

// 契約金額 500万円（不動産譲渡契約書）
result, err := jlawcore.CalcStampTax(5_000_000, date, false)
if err != nil {
    log.Fatal(err)
}

fmt.Println(result.TaxAmount)           // 2000（印紙税額）
fmt.Println(result.BracketLabel)        // 適用ブラケット名
fmt.Println(result.ReducedRateApplied)  // false

// 軽減税率適用（租税特別措置法 第91条）
// WARNING: 対象文書が軽減措置の適用要件を満たすかの事実認定は呼び出し元の責任
special, err := jlawcore.CalcStampTaxWithDocumentKind(
    1_500_000,
    date,
    true,
    jlawcore.StampTaxDocumentConstructionContract,
)
if err != nil {
    log.Fatal(err)
}
fmt.Println(special.TaxAmount)          // 200
fmt.Println(special.ReducedRateApplied) // true
```

## API リファレンス

### `CalcBrokerageFee(price uint64, year, month, day int, isLowCostVacantHouse, isSeller bool) (*BrokerageFeeResult, error)`

宅建業法第46条に基づく媒介報酬を計算する。

| 引数                   | 型       | 説明                                                             |
| ---------------------- | -------- | ---------------------------------------------------------------- |
| `price`                | `uint64` | 売買価格（円）                                                   |
| `year`                 | `int`    | 基準日（年）                                                     |
| `month`                | `int`    | 基準日（月）                                                     |
| `day`                  | `int`    | 基準日（日）                                                     |
| `isLowCostVacantHouse` | `bool`   | 低廉な空き家特例フラグ                                           |
| `isSeller`             | `bool`   | 売主側として計算するか。2018年1月〜2024年6月の特例は売主のみ適用 |

**戻り値: `*BrokerageFeeResult`**

| フィールド              | 型                | 説明                           |
| ----------------------- | ----------------- | ------------------------------ |
| `TotalWithoutTax`       | `uint64`          | 税抜合計額（円）               |
| `TotalWithTax`          | `uint64`          | 税込合計額（円）               |
| `TaxAmount`             | `uint64`          | 消費税額（円）                 |
| `LowCostSpecialApplied` | `bool`            | 低廉な空き家特例が適用されたか |
| `Breakdown`             | `[]BreakdownStep` | 各ティアの計算内訳             |

`BreakdownStep` フィールド: `Label string`, `BaseAmount uint64`, `RateNumer uint64`, `RateDenom uint64`, `Result uint64`

**エラー** — 売買価格が不正、または対象日に有効な法令パラメータが存在しない場合。

---

### `CalcIncomeTax(taxableIncome uint64, year, month, day int, applyReconstructionTax bool) (*IncomeTaxResult, error)`

所得税法第89条に基づく所得税額を計算する。

| 引数                     | 型       | 説明                                        |
| ------------------------ | -------- | ------------------------------------------- |
| `taxableIncome`          | `uint64` | 課税所得金額（円・1,000円未満切り捨て済み） |
| `year`                   | `int`    | 対象年度（年）                              |
| `month`                  | `int`    | 基準日（月）                                |
| `day`                    | `int`    | 基準日（日）                                |
| `applyReconstructionTax` | `bool`   | 復興特別所得税を適用するか                  |

**戻り値: `*IncomeTaxResult`**

| フィールド                 | 型                | 説明                                |
| -------------------------- | ----------------- | ----------------------------------- |
| `BaseTax`                  | `uint64`          | 基準所得税額（円）                  |
| `ReconstructionTax`        | `uint64`          | 復興特別所得税額（円）              |
| `TotalTax`                 | `uint64`          | 申告納税額（円・100円未満切り捨て） |
| `ReconstructionTaxApplied` | `bool`            | 復興特別所得税が適用されたか        |
| `Breakdown`                | `[]IncomeTaxStep` | 速算表の計算内訳                    |

`IncomeTaxStep` フィールド: `Label string`, `TaxableIncome uint64`, `RateNumer uint64`, `RateDenom uint64`, `Deduction uint64`, `Result uint64`

**エラー** — 課税所得金額が不正、または対象日に有効な法令パラメータが存在しない場合。

---

### `CalcStampTax(contractAmount uint64, date time.Time, isReducedRateApplicable bool) (*StampTaxResult, error)`

印紙税法 別表第一（第1号文書）に基づく印紙税額を計算する。

| 引数                      | 型       | 説明               |
| ------------------------- | -------- | ------------------ |
| `contractAmount`          | `uint64` | 契約金額（円）     |
| `date`                    | `time.Time` | 契約書作成日    |
| `isReducedRateApplicable` | `bool`   | 軽減税率適用フラグ |

**戻り値: `*StampTaxResult`**

| フィールド           | 型       | 説明                         |
| -------------------- | -------- | ---------------------------- |
| `TaxAmount`          | `uint64` | 印紙税額（円）               |
| `BracketLabel`       | `string` | 適用されたブラケットの表示名 |
| `ReducedRateApplied` | `bool`   | 軽減税率が適用されたか       |

**エラー** — 契約金額が不正、または対象日に有効な法令パラメータが存在しない場合。

### `CalcStampTaxWithDocumentKind(contractAmount uint64, date time.Time, isReducedRateApplicable bool, documentKind StampTaxDocumentKind) (*StampTaxResult, error)`

印紙税法 別表第一（第1号文書・第2号文書）に基づく印紙税額を計算する。

`documentKind` には `StampTaxDocumentRealEstateTransfer` または
`StampTaxDocumentConstructionContract` を指定する。

## ビルドとテスト

```sh
cd crates/j-law-go

# Rust staticlib をビルドしてから Go テストを実行（推奨）
make test

# リリースビルドでテスト
make test-release

# Rust staticlib のみビルド
make build-rust
```

## ライセンス

[MIT License](../../LICENSE)

/**
 * 所得税法第89条に基づく所得税計算のテスト。
 *
 * 法的根拠: 所得税法 第89条第1項 / 復興財源確保法 第13条
 * テストケースは tests/fixtures/income_tax.json から読み込む。
 *
 * 実行方法:
 *   wasm-pack build --target nodejs crates/j-law-wasm
 *   node --test crates/j-law-wasm/tests/income_tax.test.mjs
 */

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const {
  calcIncomeDeductions,
  calcIncomeTax,
  calcIncomeTaxAssessment,
} = require("../pkg/j_law_wasm.js");

const fixtures = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/income_tax.json"), "utf-8")
);
const deductionFixtures = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/income_tax_deductions.json"), "utf-8")
);

function withDate(input) {
  const [year, month, day] = input.date.split("-").map(Number);
  return {
    ...input,
    date: new Date(Date.UTC(year, month - 1, day)),
  };
}

function asBigInt(value) {
  return BigInt(value);
}

// ─── データ駆動テスト ───────────────────────────────────────────────────────

describe("calcIncomeTax - フィクスチャ駆動", () => {
  for (const c of fixtures.income_tax) {
    it(`${c.id}: ${c.description}`, () => {
      const { taxable_income, apply_reconstruction_tax } = c.input;
      const date = withDate(c.input).date;
      const r = calcIncomeTax(taxable_income, date, apply_reconstruction_tax);
      const exp = c.expected;

      assert.equal(r.baseTax, exp.base_tax, "baseTax");
      assert.equal(r.reconstructionTax, exp.reconstruction_tax, "reconstructionTax");
      assert.equal(r.totalTax, exp.total_tax, "totalTax");
      assert.equal(r.reconstructionTaxApplied, exp.reconstruction_tax_applied, "reconstructionTaxApplied");
    });
  }
});

// ─── 言語固有テスト ─────────────────────────────────────────────────────────

describe("calcIncomeTax - JS固有テスト", () => {
  it("対象日が範囲外の場合にエラー", () => {
    assert.throws(() => calcIncomeTax(5_000_000, new Date(Date.UTC(1988, 11, 31)), true));
  });

  it("breakdown の各フィールドが有効", () => {
    const r = calcIncomeTax(5_000_000, new Date(Date.UTC(2024, 0, 1)), true);
    const bd = r.breakdown();
    assert.ok(bd.length > 0, "breakdown must not be empty");
    for (const step of bd) {
      assert.ok(step.label, "label must not be empty");
      assert.ok(step.rateDenom > 0, "rateDenom must be > 0");
    }
  });
});

describe("calcIncomeDeductions - フィクスチャ駆動", () => {
  for (const c of deductionFixtures.income_tax_deductions) {
    it(`${c.id}: ${c.description}`, () => {
      const r = calcIncomeDeductions(withDate(c.input));
      const exp = c.expected;

      assert.equal(r.totalIncomeAmount, asBigInt(exp.total_income_amount), "totalIncomeAmount");
      assert.equal(r.totalDeductions, asBigInt(exp.total_deductions), "totalDeductions");
      assert.equal(
        r.taxableIncomeBeforeTruncation,
        asBigInt(exp.taxable_income_before_truncation),
        "taxableIncomeBeforeTruncation"
      );
      assert.equal(r.taxableIncome, asBigInt(exp.taxable_income), "taxableIncome");
    });
  }
});

describe("calcIncomeDeductions - JS固有テスト", () => {
  it("dependent count が u16 上限を超える場合はエラー", () => {
    assert.throws(
      () =>
        calcIncomeDeductions(
          withDate({
            total_income_amount: 5_000_000,
            date: "2024-01-01",
            dependent: {
              general_count: 70_000,
            },
          })
        ),
      /generalCount|general_count/
    );
  });

  it("u32 を超える金額を BigInt で返す", () => {
    const r = calcIncomeDeductions(
      withDate({
        total_income_amount: 10_000_000_000,
        date: "2024-01-01",
        social_insurance_premium_paid: 5_000_000_000,
      })
    );

    assert.equal(r.totalIncomeAmount, 10_000_000_000n);
    assert.equal(r.totalDeductions, 5_000_000_000n);
    assert.equal(r.taxableIncomeBeforeTruncation, 5_000_000_000n);
    assert.equal(r.taxableIncome, 5_000_000_000n);

    const breakdown = r.breakdown();
    const socialInsurance = breakdown.find((line) => line.amount === 5_000_000_000n);
    assert.ok(socialInsurance, "social insurance deduction line must exist");
    assert.equal(typeof socialInsurance.amount, "bigint");
  });
});

describe("calcIncomeTaxAssessment - フィクスチャ駆動", () => {
  for (const c of deductionFixtures.income_tax_assessment) {
    it(`${c.id}: ${c.description}`, () => {
      const r = calcIncomeTaxAssessment(
        withDate(c.input),
        c.input.apply_reconstruction_tax
      );
      const exp = c.expected;

      assert.equal(r.taxableIncome, asBigInt(exp.taxable_income), "taxableIncome");
      assert.equal(r.baseTax, asBigInt(exp.base_tax), "baseTax");
      assert.equal(r.reconstructionTax, asBigInt(exp.reconstruction_tax), "reconstructionTax");
      assert.equal(r.totalTax, asBigInt(exp.total_tax), "totalTax");
    });
  }
});

describe("calcIncomeTaxAssessment - JS固有テスト", () => {
  it("dependent count が u16 上限を超える場合はエラー", () => {
    assert.throws(
      () =>
        calcIncomeTaxAssessment(
          withDate({
            total_income_amount: 5_000_000,
            date: "2024-01-01",
            dependent: {
              specific_count: 70_000,
            },
          }),
          true
        ),
      /specificCount|specific_count/
    );
  });

  it("assessment の高額結果を BigInt で返す", () => {
    const r = calcIncomeTaxAssessment(
      withDate({
        total_income_amount: 10_000_000_000,
        date: "2024-01-01",
      }),
      true
    );

    assert.equal(r.totalIncomeAmount, 10_000_000_000n);
    assert.equal(r.taxableIncome, 10_000_000_000n);
    assert.equal(r.baseTax, 4_495_204_000n);
    assert.equal(r.reconstructionTax, 94_399_284n);
    assert.equal(r.totalTax, 4_589_603_200n);

    const taxBreakdown = r.taxBreakdown();
    assert.equal(typeof taxBreakdown[0].taxableIncome, "bigint");
    assert.equal(typeof taxBreakdown[0].result, "bigint");
    assert.equal(taxBreakdown[0].taxableIncome, 10_000_000_000n);
    assert.equal(taxBreakdown[0].result, 4_495_204_000n);
  });
});

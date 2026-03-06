/**
 * 宅建業法第46条に基づく媒介報酬計算のテスト。
 *
 * 法的根拠: 宅地建物取引業法 第46条第1項 / 国土交通省告示
 * テストケースは tests/fixtures/real_estate.json から読み込む。
 *
 * 実行方法:
 *   wasm-pack build --target nodejs crates/j-law-wasm
 *   node --test crates/j-law-wasm/tests/real_estate.test.mjs
 */

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const { calcBrokerageFee } = require("../pkg/j_law_wasm.js");

const fixtures = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/real_estate.json"), "utf-8")
);

// ─── データ駆動テスト ───────────────────────────────────────────────────────

describe("calcBrokerageFee - フィクスチャ駆動", () => {
  for (const c of fixtures.brokerage_fee) {
    it(`${c.id}: ${c.description}`, () => {
      const { price, is_low_cost_vacant_house } = c.input;
      const is_seller = c.input.is_seller ?? false;
      const [year, month, day] = c.input.date.split("-").map(Number);
      const r = calcBrokerageFee(price, year, month, day, is_low_cost_vacant_house, is_seller);
      const exp = c.expected;

      if ("total_without_tax" in exp) {
        assert.equal(r.totalWithoutTax, exp.total_without_tax, "totalWithoutTax");
      }
      if ("tax_amount" in exp) {
        assert.equal(r.taxAmount, exp.tax_amount, "taxAmount");
      }
      if ("total_with_tax" in exp) {
        assert.equal(r.totalWithTax, exp.total_with_tax, "totalWithTax");
      }
      if ("low_cost_special_applied" in exp) {
        assert.equal(r.lowCostSpecialApplied, exp.low_cost_special_applied, "lowCostSpecialApplied");
      }
      if ("breakdown_results" in exp) {
        const bd = r.breakdown();
        const actual = bd.map((s) => s.result);
        assert.deepEqual(actual, exp.breakdown_results, "breakdown_results");
      }
    });
  }
});

// ─── 言語固有テスト ─────────────────────────────────────────────────────────

describe("calcBrokerageFee - JS固有テスト", () => {
  it("対象日が範囲外の場合にエラー", () => {
    // 2018年以前はカバー範囲外（2018-01-01 が施行日のため 2017-12-31 はエラー）
    assert.throws(
      () => calcBrokerageFee(5_000_000, 2017, 12, 31, false, false),
      /2017-12-31/
    );
  });

  it("breakdown の各フィールドが有効", () => {
    const r = calcBrokerageFee(5_000_000, 2024, 8, 1, false, false);
    const bd = r.breakdown();
    for (const step of bd) {
      assert.ok(step.label, "label must not be empty");
      assert.ok(step.rateDenom > 0, "rateDenom must be > 0");
    }
  });
});

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
const { calcIncomeTax } = require("../pkg/j_law_wasm.js");

const fixtures = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/income_tax.json"), "utf-8")
);

// ─── データ駆動テスト ───────────────────────────────────────────────────────

describe("calcIncomeTax - フィクスチャ駆動", () => {
  for (const c of fixtures.income_tax) {
    it(`${c.id}: ${c.description}`, () => {
      const { taxable_income, apply_reconstruction_tax } = c.input;
      const [year, month, day] = c.input.date.split("-").map(Number);
      const date = new Date(Date.UTC(year, month - 1, day));
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
    assert.throws(() => calcIncomeTax(5_000_000, new Date(Date.UTC(2014, 11, 31)), true));
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

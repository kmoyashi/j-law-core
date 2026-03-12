/**
 * 印紙税法 別表第一に基づく印紙税額計算のテスト。
 *
 * 法的根拠: 印紙税法 別表第一 第1号文書 / 第2号文書 / 租税特別措置法 第91条
 * テストケースは tests/fixtures/stamp_tax.json から読み込む。
 *
 * 実行方法:
 *   wasm-pack build --target nodejs crates/j-law-wasm
 *   node --test crates/j-law-wasm/tests/stamp_tax.test.mjs
 */

import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const { calcStampTax } = require("../pkg/j_law_wasm.js");

const fixtures = JSON.parse(
  readFileSync(resolve(__dirname, "../../../tests/fixtures/stamp_tax.json"), "utf-8")
);

// ─── データ駆動テスト ───────────────────────────────────────────────────────

describe("calcStampTax - フィクスチャ駆動", () => {
  for (const c of fixtures.stamp_tax) {
    it(`${c.id}: ${c.description}`, () => {
      const { contract_amount, is_reduced_rate_applicable, document_kind } = c.input;
      const [year, month, day] = c.input.date.split("-").map(Number);
      const date = new Date(Date.UTC(year, month - 1, day));
      const r = calcStampTax(contract_amount, date, is_reduced_rate_applicable, document_kind);
      const exp = c.expected;

      assert.equal(r.taxAmount, exp.tax_amount, "taxAmount");
      assert.equal(r.reducedRateApplied, exp.reduced_rate_applied, "reducedRateApplied");
    });
  }
});

// ─── 言語固有テスト ─────────────────────────────────────────────────────────

describe("calcStampTax - JS固有テスト", () => {
  it("対象日が範囲外の場合にエラー", () => {
    assert.throws(() => calcStampTax(5_000_000, new Date(Date.UTC(2014, 2, 31)), false));
  });

  it("bracket_label が返される", () => {
    const r = calcStampTax(5_000_000, new Date(Date.UTC(2024, 7, 1)), false);
    assert.ok(r.bracketLabel, "bracketLabel must not be empty");
  });

  it("不正な documentKind を拒否する", () => {
    assert.throws(() =>
      calcStampTax(5_000_000, new Date(Date.UTC(2024, 7, 1)), false, "invalid_kind")
    );
  });
});

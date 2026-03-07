use std::os::raw::{c_char, c_int, c_long};

use j_law_uniffi::{
    calc_brokerage_fee as uniffi_calc_brokerage_fee,
    calc_consumption_tax as uniffi_calc_consumption_tax, calc_income_tax as uniffi_calc_income_tax,
    calc_stamp_tax as uniffi_calc_stamp_tax,
};
use rb_sys::*;

// ── ヘルパー ──────────────────────────────────────────────────────────────────

/// `name` をキーとする Ruby Symbol を生成する。
macro_rules! sym {
    ($name:literal) => {
        rb_id2sym(rb_intern(concat!($name, "\0").as_ptr() as *const c_char))
    };
}

/// Rust `&str` から Ruby String を生成する。
unsafe fn ruby_str(s: &str) -> VALUE {
    rb_str_new(s.as_ptr() as *const c_char, s.len() as c_long)
}

/// `u64` を Ruby Integer に変換する。
///
/// 実用的な税額・価格は isize::MAX を超えないため、キャストは安全。
unsafe fn ruby_u64(n: u64) -> VALUE {
    rb_int2inum(n as isize)
}

/// `bool` を Ruby Integer（0 or 1）に変換する。
///
/// Ruby 側では整数として受け取り、`== 1` で boolean に変換する。
unsafe fn ruby_bool_int(b: bool) -> VALUE {
    rb_int2inum(if b { 1 } else { 0 })
}

/// `argv[idx]` から Ruby Integer を `u64` として取り出す。
unsafe fn arg_u64(argv: *const VALUE, idx: usize) -> u64 {
    rb_num2long(*argv.add(idx)) as u64
}

/// `argv[idx]` から Ruby Integer を `u64` として取り出す（年月日など小さい値用）。
unsafe fn arg_ulong(argv: *const VALUE, idx: usize) -> u64 {
    rb_num2long(*argv.add(idx)) as u64
}

/// `argv[idx]` から 0/1 整数を bool として取り出す。
unsafe fn arg_bool(argv: *const VALUE, idx: usize) -> bool {
    rb_num2long(*argv.add(idx)) != 0
}

/// RuntimeError を発生させる。
///
/// `rb_exc_raise` は NORETURN（`-> !`）のため、この関数も発散する。
unsafe fn raise_runtime(msg: &str) -> ! {
    let rb_msg = ruby_str(msg);
    // rb_str_new が msg の内容を Ruby ヒープにコピー済み
    let exc = rb_exc_new_str(rb_eRuntimeError, rb_msg);
    rb_exc_raise(exc)
}

/// ArgError を発生させる。
unsafe fn raise_arg_error(msg: &str) -> ! {
    let rb_msg = ruby_str(msg);
    let exc = rb_exc_new_str(rb_eArgError, rb_msg);
    rb_exc_raise(exc)
}

// ── 消費税計算 ────────────────────────────────────────────────────────────────

/// JLawRubyNative.calc_consumption_tax(amount, year, month, day, is_reduced_rate_int) -> Hash
///
/// is_reduced_rate_int: 0 or 1
unsafe extern "C" fn calc_consumption_tax_native(
    argc: c_int,
    argv: *const VALUE,
    _recv: VALUE,
) -> VALUE {
    if argc != 5 {
        raise_arg_error("wrong number of arguments");
    }

    let amount = arg_u64(argv, 0);
    let year = arg_ulong(argv, 1) as u16;
    let month = arg_ulong(argv, 2) as u8;
    let day = arg_ulong(argv, 3) as u8;
    let is_reduced_rate = arg_bool(argv, 4);

    match uniffi_calc_consumption_tax(amount, year, month, day, is_reduced_rate) {
        Ok(r) => {
            let hash = rb_hash_new();
            rb_hash_aset(hash, sym!("tax_amount"), ruby_u64(r.tax_amount));
            rb_hash_aset(hash, sym!("amount_with_tax"), ruby_u64(r.amount_with_tax));
            rb_hash_aset(
                hash,
                sym!("amount_without_tax"),
                ruby_u64(r.amount_without_tax),
            );
            rb_hash_aset(
                hash,
                sym!("applied_rate_numer"),
                ruby_u64(r.applied_rate_numer),
            );
            rb_hash_aset(
                hash,
                sym!("applied_rate_denom"),
                ruby_u64(r.applied_rate_denom),
            );
            rb_hash_aset(
                hash,
                sym!("is_reduced_rate"),
                ruby_bool_int(r.is_reduced_rate),
            );
            hash
        }
        Err(e) => raise_runtime(&e.to_string()),
    }
}

// ── 媒介報酬計算 ──────────────────────────────────────────────────────────────

/// JLawRubyNative.calc_brokerage_fee(price, year, month, day, is_low_cost_int, is_seller_int) -> Hash
unsafe extern "C" fn calc_brokerage_fee_native(
    argc: c_int,
    argv: *const VALUE,
    _recv: VALUE,
) -> VALUE {
    if argc != 6 {
        raise_arg_error("wrong number of arguments");
    }

    let price = arg_u64(argv, 0);
    let year = arg_ulong(argv, 1) as u16;
    let month = arg_ulong(argv, 2) as u8;
    let day = arg_ulong(argv, 3) as u8;
    let is_low_cost_vacant_house = arg_bool(argv, 4);
    let is_seller = arg_bool(argv, 5);

    match uniffi_calc_brokerage_fee(price, year, month, day, is_low_cost_vacant_house, is_seller) {
        Ok(r) => {
            let hash = rb_hash_new();
            rb_hash_aset(
                hash,
                sym!("total_without_tax"),
                ruby_u64(r.total_without_tax),
            );
            rb_hash_aset(hash, sym!("total_with_tax"), ruby_u64(r.total_with_tax));
            rb_hash_aset(hash, sym!("tax_amount"), ruby_u64(r.tax_amount));
            rb_hash_aset(
                hash,
                sym!("low_cost_special_applied"),
                ruby_bool_int(r.low_cost_special_applied),
            );

            let breakdown_ary = rb_ary_new();
            for step in r.breakdown.iter() {
                let h = rb_hash_new();
                rb_hash_aset(h, sym!("label"), ruby_str(&step.label));
                rb_hash_aset(h, sym!("base_amount"), ruby_u64(step.base_amount));
                rb_hash_aset(h, sym!("rate_numer"), ruby_u64(step.rate_numer));
                rb_hash_aset(h, sym!("rate_denom"), ruby_u64(step.rate_denom));
                rb_hash_aset(h, sym!("result"), ruby_u64(step.result));
                rb_ary_push(breakdown_ary, h);
            }
            rb_hash_aset(hash, sym!("breakdown"), breakdown_ary);
            hash
        }
        Err(e) => raise_runtime(&e.to_string()),
    }
}

// ── 所得税計算 ────────────────────────────────────────────────────────────────

/// JLawRubyNative.calc_income_tax(taxable_income, year, month, day, apply_reconstruction_int) -> Hash
unsafe extern "C" fn calc_income_tax_native(
    argc: c_int,
    argv: *const VALUE,
    _recv: VALUE,
) -> VALUE {
    if argc != 5 {
        raise_arg_error("wrong number of arguments");
    }

    let taxable_income = arg_u64(argv, 0);
    let year = arg_ulong(argv, 1) as u16;
    let month = arg_ulong(argv, 2) as u8;
    let day = arg_ulong(argv, 3) as u8;
    let apply_reconstruction_tax = arg_bool(argv, 4);

    match uniffi_calc_income_tax(taxable_income, year, month, day, apply_reconstruction_tax) {
        Ok(r) => {
            let hash = rb_hash_new();
            rb_hash_aset(hash, sym!("base_tax"), ruby_u64(r.base_tax));
            rb_hash_aset(
                hash,
                sym!("reconstruction_tax"),
                ruby_u64(r.reconstruction_tax),
            );
            rb_hash_aset(hash, sym!("total_tax"), ruby_u64(r.total_tax));
            rb_hash_aset(
                hash,
                sym!("reconstruction_tax_applied"),
                ruby_bool_int(r.reconstruction_tax_applied),
            );

            let breakdown_ary = rb_ary_new();
            for step in r.breakdown.iter() {
                let h = rb_hash_new();
                rb_hash_aset(h, sym!("label"), ruby_str(&step.label));
                rb_hash_aset(h, sym!("taxable_income"), ruby_u64(step.taxable_income));
                rb_hash_aset(h, sym!("rate_numer"), ruby_u64(step.rate_numer));
                rb_hash_aset(h, sym!("rate_denom"), ruby_u64(step.rate_denom));
                rb_hash_aset(h, sym!("deduction"), ruby_u64(step.deduction));
                rb_hash_aset(h, sym!("result"), ruby_u64(step.result));
                rb_ary_push(breakdown_ary, h);
            }
            rb_hash_aset(hash, sym!("breakdown"), breakdown_ary);
            hash
        }
        Err(e) => raise_runtime(&e.to_string()),
    }
}

// ── 印紙税計算 ────────────────────────────────────────────────────────────────

/// JLawRubyNative.calc_stamp_tax(contract_amount, year, month, day, is_reduced_rate_applicable_int) -> Hash
unsafe extern "C" fn calc_stamp_tax_native(argc: c_int, argv: *const VALUE, _recv: VALUE) -> VALUE {
    if argc != 5 {
        raise_arg_error("wrong number of arguments");
    }

    let contract_amount = arg_u64(argv, 0);
    let year = arg_ulong(argv, 1) as u16;
    let month = arg_ulong(argv, 2) as u8;
    let day = arg_ulong(argv, 3) as u8;
    let is_reduced_rate_applicable = arg_bool(argv, 4);

    match uniffi_calc_stamp_tax(
        contract_amount,
        year,
        month,
        day,
        is_reduced_rate_applicable,
    ) {
        Ok(r) => {
            let hash = rb_hash_new();
            rb_hash_aset(hash, sym!("tax_amount"), ruby_u64(r.tax_amount));
            rb_hash_aset(hash, sym!("bracket_label"), ruby_str(&r.bracket_label));
            rb_hash_aset(
                hash,
                sym!("reduced_rate_applied"),
                ruby_bool_int(r.reduced_rate_applied),
            );
            hash
        }
        Err(e) => raise_runtime(&e.to_string()),
    }
}

// ── モジュール初期化 ──────────────────────────────────────────────────────────

/// Ruby ネイティブ拡張の初期化エントリポイント。
///
/// `JLawRubyNative` モジュールを定義し、各計算関数をモジュール関数として登録する。
/// Ruby 側の薄いラッパー（JLawRuby モジュール）がこれらを呼び出す。
///
/// # Safety
///
/// Ruby ランタイムが拡張をロードする際に呼び出す。
/// Ruby の GIL 保持下でのみ呼ばれるため安全。
#[no_mangle]
pub unsafe extern "C" fn Init_j_law_ruby() {
    let module = rb_define_module(c"JLawRubyNative".as_ptr());

    // argc = -1: Ruby が (int argc, VALUE *argv, VALUE self) でコールバックする形式。
    // rb_define_module_function の func 型 (fn() -> VALUE) とは実際には異なるが、
    // Ruby C API の ANYARGS 規約に従い transmute で変換する。
    type Callback = unsafe extern "C" fn(c_int, *const VALUE, VALUE) -> VALUE;
    type RbFunc = unsafe extern "C" fn() -> VALUE;

    rb_define_module_function(
        module,
        c"calc_consumption_tax".as_ptr(),
        Some(std::mem::transmute::<Callback, RbFunc>(
            calc_consumption_tax_native,
        )),
        -1,
    );
    rb_define_module_function(
        module,
        c"calc_brokerage_fee".as_ptr(),
        Some(std::mem::transmute::<Callback, RbFunc>(
            calc_brokerage_fee_native,
        )),
        -1,
    );
    rb_define_module_function(
        module,
        c"calc_income_tax".as_ptr(),
        Some(std::mem::transmute::<Callback, RbFunc>(
            calc_income_tax_native,
        )),
        -1,
    );
    rb_define_module_function(
        module,
        c"calc_stamp_tax".as_ptr(),
        Some(std::mem::transmute::<Callback, RbFunc>(
            calc_stamp_tax_native,
        )),
        -1,
    );
}

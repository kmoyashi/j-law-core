// j-law-uniffi の UniFFI スキャフォールディングシンボルをこの cdylib へ再エクスポートする。
//
// Ruby 側は rb-sys の Init_ エントリポイントを使わず、
// ffi gem 経由で UniFFI が生成した Ruby バインディング（j_law_uniffi.rb）が
// この .so をロードして uniffi_j_law_uniffi_fn_func_* を直接呼び出す。
j_law_uniffi::uniffi_reexport_scaffolding!();

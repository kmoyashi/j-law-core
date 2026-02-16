# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name    = "j_law_core"
  spec.version = "0.1.0"
  spec.authors = ["j-law-core contributors"]
  spec.summary = "日本の法令が定める各種計算を法的正確性を保証して実装するライブラリ"

  spec.required_ruby_version = ">= 3.0"

  spec.files = Dir[
    "lib/**/*.rb",
    "ext/**/*.rb",
    "src/**/*.rs",
    "Cargo.toml",
    "Cargo.lock",
  ]
  spec.extensions = ["ext/j_law_core/extconf.rb"]

  spec.add_dependency "rb_sys", "~> 0.9"
end

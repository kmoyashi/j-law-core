# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name    = "j_law_ruby"
  spec.version = "0.0.1"
  spec.authors = ["j-law-core contributors"]
  spec.email   = []
  spec.summary = "日本の法令が定める各種計算を法的正確性を保証して実装するライブラリ"
  spec.description = <<~DESC
    宅地建物取引業法・所得税法・印紙税法が定める各種計算を、
    整数演算による端数処理の再現性を保証して実装するRubyバインディングです。
  DESC
  spec.homepage = "https://github.com/kmoyashi/j-law-core"
  spec.license  = "MIT"

  spec.required_ruby_version = ">= 3.0"

  spec.metadata = {
    "homepage_uri"    => spec.homepage,
    "source_code_uri" => "https://github.com/kmoyashi/j-law-core/tree/main/crates/j-law-ruby",
    "changelog_uri"   => "https://github.com/kmoyashi/j-law-core/releases",
    "bug_tracker_uri" => "https://github.com/kmoyashi/j-law-core/issues",
  }

  spec.files = Dir[
    "lib/**/*.rb",
    "ext/**/*.rb",
    "rake_support/**/*.rb",
    "test/**/*.rb",
    "vendor/rust/**/*",
    "Gemfile",
    "Rakefile",
    "README.md",
  ].select { |path| File.file?(path) }
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/j_law_ruby/extconf.rb"]

  spec.add_dependency "ffi", "~> 1.0"
end

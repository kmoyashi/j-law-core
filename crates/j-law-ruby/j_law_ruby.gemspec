# frozen_string_literal: true

require_relative "lib/j_law_ruby/build_support"

Gem::Specification.new do |spec|
  binary_gem_build = JLawRuby::BuildSupport.binary_gem_build?

  spec.name    = "j-law-ruby"
  spec.version = "0.0.7"
  spec.authors = ["j-law-core contributors"]
  spec.email   = []
  spec.summary = "日本法令計算のPoCをRubyから利用するためのアルファ版バインディング"
  spec.description = <<~DESC
    日本法令計算の PoC を Ruby から利用するためのバインディングです。
    実装は alpha 段階であり、計算結果の正確性・完全性・最新性を保証するものではありません。
  DESC
  spec.homepage = "https://github.com/kmoyashi/j-law-core"
  spec.license  = "MIT"

  spec.required_ruby_version = ">= 3.1"

  spec.metadata = {
    "homepage_uri"    => spec.homepage,
    "source_code_uri" => "https://github.com/kmoyashi/j-law-core/tree/main/crates/j-law-ruby",
    "changelog_uri"   => "https://github.com/kmoyashi/j-law-core/releases",
    "bug_tracker_uri" => "https://github.com/kmoyashi/j-law-core/issues",
  }

  spec.files = Dir[
    "lib/**/*.rb",
    "README.md",
    *(binary_gem_build ? ["lib/j_law_ruby/native/*"] : [
      "ext/**/*.rb",
      "rake_support/**/*.rb",
      "test/**/*.rb",
      "vendor/rust/**/*",
      "Gemfile",
      "Rakefile",
    ]),
  ].select { |path| File.file?(path) }
  spec.require_paths = ["lib"]

  if binary_gem_build
    spec.platform = JLawRuby::BuildSupport.gem_platform
  else
    spec.extensions = ["ext/j_law_ruby/extconf.rb"]
  end

  spec.add_dependency "ffi", "~> 1.0"
end

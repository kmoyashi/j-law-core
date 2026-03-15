# frozen_string_literal: true

require "json"
require "minitest/autorun"
require "open3"
require "fileutils"
require "bundler"
require_relative "../lib/j_law_ruby/build_support"

class TestGemspec < Minitest::Test
  GEM_ROOT = File.expand_path("..", __dir__)

  def test_source_gem_keeps_ruby_platform_and_extension_build
    spec = with_native_library_placeholder { load_gemspec(binary: false) }

    assert_equal Gem::Platform::RUBY, spec.fetch("platform")
    assert_equal ["ext/j_law_ruby/extconf.rb"], spec.fetch("extensions")
    refute_includes spec.fetch("files"), native_library_path
  end

  def test_binary_gem_is_platform_specific_and_bundles_native_library
    spec = with_native_library_placeholder { load_gemspec(binary: true) }

    assert_equal Gem::Platform.local.to_s, spec.fetch("platform")
    assert_empty spec.fetch("extensions")
    assert_includes spec.fetch("files"), native_library_path
  end

  private

  def load_gemspec(binary:)
    script = <<~RUBY
      require "json"

      spec = Gem::Specification.load("j_law_ruby.gemspec")
      abort "failed to load gemspec" if spec.nil?

      puts JSON.generate(
        platform: spec.platform.to_s,
        extensions: spec.extensions,
        files: spec.files.sort
      )
    RUBY

    stdout = nil
    stderr = nil
    status = nil

    Bundler.with_unbundled_env do
      stdout, stderr, status = Open3.capture3(
        { JLawRuby::BuildSupport::BINARY_GEM_ENV => (binary ? "1" : "0") },
        "ruby",
        "-e",
        script,
        chdir: GEM_ROOT
      )
    end

    assert status.success?, stderr

    JSON.parse(stdout)
  end

  def native_library_path
    File.join(
      "lib",
      "j_law_ruby",
      "native",
      JLawRuby::BuildSupport.shared_library_filename
    )
  end

  def with_native_library_placeholder
    path = File.join(GEM_ROOT, native_library_path)
    created = !File.exist?(path)
    FileUtils.mkdir_p(File.dirname(path))
    File.write(path, "") if created
    yield
  ensure
    FileUtils.rm_f(path) if created
  end
end

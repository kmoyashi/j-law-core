# frozen_string_literal: true

require "fileutils"
require "mkmf"
require_relative "../../lib/j_law_ruby/build_support"

gem_root = File.expand_path("../..", __dir__)
packaged_library = JLawRuby::BuildSupport.packaged_shared_library_path(gem_root)

unless JLawRuby::BuildSupport.should_build_shared_library?(gem_root)
  puts "Using packaged j-law-c-ffi shared library at #{packaged_library}"
  JLawRuby::BuildSupport.write_stub_makefile("Makefile")
  exit 0
end

manifest_path = JLawRuby::BuildSupport.manifest_path(gem_root)

abort "Cargo workspace for j-law-c-ffi was not found." if manifest_path.nil?

profile = JLawRuby::BuildSupport.cargo_profile
cargo_command = ["cargo", "build", "-p", "j-law-c-ffi", "--manifest-path", manifest_path]
cargo_command << "--release" if profile == "release"

puts "Building j-law-c-ffi (#{profile}) via #{manifest_path}"
abort "cargo build failed" unless system(*cargo_command)

built_library = JLawRuby::BuildSupport.built_shared_library_path(manifest_path, profile)
abort "built shared library was not found: #{built_library}" unless File.file?(built_library)

native_dir = JLawRuby::BuildSupport.native_dir(gem_root)
FileUtils.mkdir_p(native_dir)
FileUtils.cp(built_library, packaged_library)

JLawRuby::BuildSupport.write_stub_makefile("Makefile")

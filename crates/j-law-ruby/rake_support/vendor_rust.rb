# frozen_string_literal: true

require "fileutils"

module JLawRuby
  module VendorRust
    module_function

    WORKSPACE_TOML = <<~TOML
      [workspace]
      members = [
          "crates/j-law-core",
          "crates/j-law-registry",
          "crates/j-law-ffi",
      ]
      resolver = "2"

      [workspace.lints.clippy]
      disallowed_methods = "warn"
      disallowed_types = "warn"
      disallowed_macros = "warn"
    TOML

    COPY_MAP = {
      "crates/j-law-ffi" => %w[Cargo.toml src j_law_ffi.h],
      "crates/j-law-core" => %w[Cargo.toml src],
      "crates/j-law-registry" => %w[Cargo.toml src data],
    }.freeze

    def prepare!(gem_root)
      vendor_root = File.join(gem_root, "vendor", "rust")
      repo_root = File.expand_path("../..", gem_root)

      FileUtils.rm_rf(vendor_root)
      FileUtils.mkdir_p(vendor_root)
      File.write(File.join(vendor_root, "Cargo.toml"), WORKSPACE_TOML)

      cargo_lock = File.join(repo_root, "Cargo.lock")
      FileUtils.cp(cargo_lock, File.join(vendor_root, "Cargo.lock")) if File.file?(cargo_lock)

      COPY_MAP.each do |crate_dir, entries|
        entries.each do |entry|
          source = File.join(repo_root, crate_dir, entry)
          destination = File.join(vendor_root, crate_dir, entry)
          FileUtils.mkdir_p(File.dirname(destination))
          FileUtils.cp_r(source, destination)
        end
      end
    end
  end
end

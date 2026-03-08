# frozen_string_literal: true

require "rbconfig"

module JLawRuby
  module BuildSupport
    module_function

    def cargo_profile
      ENV.fetch("JLAW_RUBY_CARGO_PROFILE", "release")
    end

    def cargo_target
      host_os = RbConfig::CONFIG["host_os"]
      host_cpu = RbConfig::CONFIG["host_cpu"]

      case host_os
      when /darwin/
        return "x86_64-apple-darwin" if host_cpu == "x86_64"
        return "aarch64-apple-darwin" if %w[aarch64 arm64].include?(host_cpu)
      when /linux/
        return "x86_64-unknown-linux-gnu" if host_cpu == "x86_64"
        return "aarch64-unknown-linux-gnu" if %w[aarch64 arm64].include?(host_cpu)
      end

      nil
    end

    def shared_library_filename
      case RbConfig::CONFIG["host_os"]
      when /mswin|mingw|cygwin/
        "j_law_ffi.dll"
      when /darwin/
        "libj_law_ffi.dylib"
      else
        "libj_law_ffi.so"
      end
    end

    def native_dir(gem_root)
      File.join(gem_root, "lib", "j_law_ruby", "native")
    end

    def packaged_shared_library_path(gem_root)
      File.join(native_dir(gem_root), shared_library_filename)
    end

    def vendored_workspace_root(gem_root)
      File.join(gem_root, "vendor", "rust")
    end

    def vendored_manifest_path(gem_root)
      File.join(vendored_workspace_root(gem_root), "Cargo.toml")
    end

    def repo_workspace_root(gem_root)
      File.expand_path("../..", gem_root)
    end

    def repo_manifest_path(gem_root)
      File.join(repo_workspace_root(gem_root), "Cargo.toml")
    end

    def manifest_path(gem_root)
      vendored_manifest = vendored_manifest_path(gem_root)
      return vendored_manifest if File.file?(vendored_manifest)

      repo_manifest = repo_manifest_path(gem_root)
      return repo_manifest if File.file?(repo_manifest)

      nil
    end

    def built_shared_library_path(manifest_path, profile, target = cargo_target)
      target_components = ["target"]
      target_components << target unless target.nil? || target.empty?
      target_components << profile

      File.join(File.dirname(manifest_path), *target_components, shared_library_filename)
    end

    def shared_library_candidates(gem_root)
      candidates = []
      env_path = ENV["JLAW_RUBY_FFI_LIB"]
      candidates << env_path unless env_path.nil? || env_path.empty?
      candidates << packaged_shared_library_path(gem_root)

      [vendored_manifest_path(gem_root), repo_manifest_path(gem_root)].each do |manifest|
        next unless File.file?(manifest)

        candidates << built_shared_library_path(manifest, "release", nil)
        candidates << built_shared_library_path(manifest, "debug", nil)
        candidates << built_shared_library_path(manifest, "release")
        candidates << built_shared_library_path(manifest, "debug")
      end

      candidates.uniq
    end

    def resolve_shared_library_path(gem_root)
      shared_library_candidates(gem_root).find { |path| File.file?(path) }
    end
  end
end

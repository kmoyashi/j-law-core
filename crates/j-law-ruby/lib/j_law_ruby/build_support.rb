# frozen_string_literal: true

require "rbconfig"

module JLawRuby
  module BuildSupport
    module_function

    BINARY_GEM_ENV = "JLAW_RUBY_BINARY_GEM"
    FORCE_CARGO_BUILD_ENV = "JLAW_RUBY_FORCE_CARGO_BUILD"

    def cargo_profile
      ENV.fetch("JLAW_RUBY_CARGO_PROFILE", "release")
    end

    def binary_gem_build?
      ENV[BINARY_GEM_ENV] == "1"
    end

    def force_cargo_build?
      ENV[FORCE_CARGO_BUILD_ENV] == "1"
    end

    def gem_platform
      require "rubygems/platform"

      Gem::Platform.local
    end

    def make_command
      RbConfig::CONFIG.fetch("MAKE", "make")
    end

    def shared_library_filename
      case RbConfig::CONFIG["host_os"]
      when /mswin|mingw|cygwin/
        "j_law_c_ffi.dll"
      when /darwin/
        "libj_law_c_ffi.dylib"
      else
        "libj_law_c_ffi.so"
      end
    end

    def native_dir(gem_root)
      File.join(gem_root, "lib", "j_law_ruby", "native")
    end

    def packaged_shared_library_path(gem_root)
      File.join(native_dir(gem_root), shared_library_filename)
    end

    def packaged_shared_library?(gem_root)
      File.file?(packaged_shared_library_path(gem_root))
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

    def built_shared_library_path(manifest_path, profile)
      File.join(File.dirname(manifest_path), "target", profile, shared_library_filename)
    end

    def shared_library_candidates(gem_root)
      candidates = []
      env_path = ENV["JLAW_RUBY_C_FFI_LIB"]
      candidates << env_path unless env_path.nil? || env_path.empty?
      candidates << packaged_shared_library_path(gem_root)

      [vendored_manifest_path(gem_root), repo_manifest_path(gem_root)].each do |manifest|
        next unless File.file?(manifest)

        candidates << built_shared_library_path(manifest, "release")
        candidates << built_shared_library_path(manifest, "debug")
      end

      candidates.uniq
    end

    def resolve_shared_library_path(gem_root)
      shared_library_candidates(gem_root).find { |path| File.file?(path) }
    end

    def should_build_shared_library?(gem_root)
      return true if force_cargo_build?

      !packaged_shared_library?(gem_root)
    end

    def write_stub_makefile(path = "Makefile")
      File.write(
        path,
        <<~MAKEFILE
          all:
          \t@true

          install:
          \t@true

          clean:
          \t@true
        MAKEFILE
      )
    end
  end
end

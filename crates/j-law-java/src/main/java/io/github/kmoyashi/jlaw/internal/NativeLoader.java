package io.github.kmoyashi.jlaw.internal;

import io.github.kmoyashi.jlaw.JLawException;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Locale;

public final class NativeLoader {
    static final String OVERRIDE_PROPERTY = "jlaw.native.lib";
    static final String OVERRIDE_ENV = "JLAW_JAVA_NATIVE_LIB";

    private static final Object LOCK = new Object();
    private static volatile boolean loaded;

    private NativeLoader() {
    }

    public static void load() {
        if (loaded) {
            return;
        }

        synchronized (LOCK) {
            if (loaded) {
                return;
            }

            String overridePath = System.getProperty(OVERRIDE_PROPERTY);
            if (overridePath == null || overridePath.isEmpty()) {
                overridePath = System.getenv(OVERRIDE_ENV);
            }

            if (overridePath != null && !overridePath.isEmpty()) {
                System.load(overridePath);
                loaded = true;
                return;
            }

            Platform platform = detectPlatform(System.getProperty("os.name"), System.getProperty("os.arch"));
            String resourcePath = "META-INF/jlaw/native/" + platform.getOs() + "/" + platform.getArch() + "/" + platform.libraryFileName();
            try (InputStream inputStream = NativeLoader.class.getClassLoader().getResourceAsStream(resourcePath)) {
                if (inputStream == null) {
                    throw new JLawException("Native library not found for " + platform + " at " + resourcePath);
                }

                Path tempDir = Files.createTempDirectory("jlaw-native-");
                Path extracted = tempDir.resolve(platform.libraryFileName());
                Files.copy(inputStream, extracted, StandardCopyOption.REPLACE_EXISTING);
                tempDir.toFile().deleteOnExit();
                extracted.toFile().deleteOnExit();
                System.load(extracted.toAbsolutePath().toString());
                loaded = true;
            } catch (IOException e) {
                throw new JLawException("Failed to extract native library", e);
            }
        }
    }

    static Platform detectPlatform(String osName, String osArch) {
        String normalizedOs = normalizeOs(osName);
        String normalizedArch = normalizeArch(osArch);
        return new Platform(normalizedOs, normalizedArch);
    }

    static String normalizeOs(String osName) {
        String value = osName == null ? "" : osName.toLowerCase(Locale.ROOT);
        if (value.contains("mac") || value.contains("darwin")) {
            return "macos";
        }
        if (value.contains("win")) {
            return "windows";
        }
        if (value.contains("nux") || value.contains("linux")) {
            return "linux";
        }
        throw new JLawException("Unsupported operating system: " + osName);
    }

    static String normalizeArch(String osArch) {
        String value = osArch == null ? "" : osArch.toLowerCase(Locale.ROOT);
        if ("x86_64".equals(value) || "amd64".equals(value)) {
            return "x86_64";
        }
        if ("aarch64".equals(value) || "arm64".equals(value)) {
            return "aarch64";
        }
        throw new JLawException("Unsupported architecture: " + osArch);
    }

    static final class Platform {
        private final String os;
        private final String arch;

        Platform(String os, String arch) {
            this.os = os;
            this.arch = arch;
        }

        String getOs() {
            return os;
        }

        String getArch() {
            return arch;
        }

        String libraryFileName() {
            if ("windows".equals(os)) {
                return "jlaw_jni.dll";
            }
            if ("macos".equals(os)) {
                return "libjlaw_jni.dylib";
            }
            return "libjlaw_jni.so";
        }

        @Override
        public String toString() {
            return os + "/" + arch;
        }
    }
}

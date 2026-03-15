package io.github.kmoyashi.jlaw.internal;

import io.github.kmoyashi.jlaw.JLawException;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class NativeLoaderTest {
    @Test
    void normalizesKnownPlatforms() {
        assertEquals("macos", NativeLoader.normalizeOs("Mac OS X"));
        assertEquals("windows", NativeLoader.normalizeOs("Windows 11"));
        assertEquals("linux", NativeLoader.normalizeOs("Linux"));
        assertEquals("aarch64", NativeLoader.normalizeArch("arm64"));
        assertEquals("x86_64", NativeLoader.normalizeArch("amd64"));
    }

    @Test
    void rejectsUnsupportedPlatformValues() {
        assertThrows(JLawException.class, () -> NativeLoader.normalizeOs("Solaris"));
        assertThrows(JLawException.class, () -> NativeLoader.normalizeArch("sparc"));
    }

    @Test
    void loadIsIdempotent() {
        NativeLoader.load();
        NativeLoader.load();
    }
}

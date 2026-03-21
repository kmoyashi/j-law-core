package jlawcore

import "testing"

func TestVerifyFFIVersion(t *testing.T) {
	if err := verifyFFIVersion(); err != nil {
		t.Fatalf("unexpected ffi version mismatch: %v", err)
	}
}

func TestBoundedLengthClampsOutOfRangeValues(t *testing.T) {
	if got := boundedLength(-1, 8); got != 0 {
		t.Fatalf("boundedLength should clamp negative values to 0, got %d", got)
	}

	if got := boundedLength(99, 8); got != 8 {
		t.Fatalf("boundedLength should clamp oversized values to max, got %d", got)
	}
}

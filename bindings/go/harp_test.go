package harp

import (
	"strings"
	"testing"
)

func TestGenerateName(t *testing.T) {
	name := GenerateName()
	parts := strings.Split(name, "-")
	if len(parts) != 3 {
		t.Errorf("GenerateName() = %q; want 3 parts separated by dashes", name)
	}
}

func TestGenerateNameProducesDifferentResults(t *testing.T) {
	seen := make(map[string]bool)
	for i := 0; i < 10; i++ {
		name := GenerateName()
		seen[name] = true
	}
	if len(seen) < 2 {
		t.Error("GenerateName() should produce varied names")
	}
}

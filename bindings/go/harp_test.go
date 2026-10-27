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

func TestGenerateNameWithOptionsFourComponents(t *testing.T) {
	name, err := GenerateNameWithOptions(Options{Components: 4, Separator: "-"})
	if err != nil {
		t.Fatalf("4 components refused: %v", err)
	}
	if got := len(strings.Split(name, "-")); got != 4 {
		t.Errorf("GenerateNameWithOptions = %q; want 4 parts", name)
	}
}

func TestGenerateNameWithOptionsRefusesWhatTheCoreRefuses(t *testing.T) {
	cases := []struct {
		name string
		opts Options
		want string
	}{
		{"five components", Options{Components: 5, Separator: "-"}, "components must be 2..=4, got 5"},
		{"one component", Options{Components: 1, Separator: "-"}, "components must be 2..=4, got 1"},
		{"an emptying filter", Options{Components: 2, MaxElementLength: 2, Separator: "-"}, "max_element_length 2 leaves an empty word list"},
		{"an empty separator", Options{Components: 2, Separator: ""}, "cannot be split unambiguously"},
		{"a letter separator", Options{Components: 2, Separator: "x"}, "cannot be split unambiguously"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			name, err := GenerateNameWithOptions(c.opts)
			if err == nil {
				t.Fatalf("got %q; want a refusal", name)
			}
			if !strings.Contains(err.Error(), c.want) {
				t.Errorf("error %q; want it to contain %q", err, c.want)
			}
		})
	}
}

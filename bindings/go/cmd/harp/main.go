// harp generates random human-readable names from adjectives and nouns.
package main

import (
	"flag"
	"fmt"
	"os"

	harp "github.com/benjaminabbitt/harp/bindings/go"
)

func main() {
	components := flag.Uint("c", 3, "number of components (2-4)")
	maxLen := flag.Uint("m", 0, "max length per word (0 = no limit)")
	separator := flag.String("s", "-", "separator between words")
	version := flag.Bool("version", false, "print version and exit")
	flag.Parse()

	if *version {
		fmt.Println(harp.Version())
		return
	}

	// The library refuses components outside 2-4; check here too, so a value
	// past 255 is not truncated into range by the uint8 conversion.
	if *components < 2 || *components > 4 {
		fmt.Fprintln(os.Stderr, "error: components must be between 2 and 4")
		os.Exit(1)
	}

	opts := harp.Options{
		Components:       uint8(*components),
		MaxElementLength: *maxLen,
		Separator:        *separator,
	}

	name, err := harp.GenerateNameWithOptions(opts)
	if err != nil {
		fmt.Fprintln(os.Stderr, "error:", err)
		os.Exit(1)
	}
	fmt.Println(name)
}

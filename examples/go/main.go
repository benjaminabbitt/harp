// CLI example using harp-ffi bindings.
package main

/*
#cgo LDFLAGS: -L${SRCDIR}/../../target/release -lharp_ffi
#include <stdlib.h>

extern char* harp_generate_name(void);
extern void harp_free_string(char* s);
extern const char* harp_version(void);
*/
import "C"

import (
	"flag"
	"fmt"
	"os"
)

func generateName() string {
	cstr := C.harp_generate_name()
	defer C.harp_free_string(cstr)
	return C.GoString(cstr)
}

func version() string {
	return C.GoString(C.harp_version())
}

func main() {
	count := flag.Int("n", 1, "Number of names to generate")
	showVersion := flag.Bool("v", false, "Show version and exit")
	flag.Parse()

	if *showVersion {
		fmt.Printf("harp %s\n", version())
		os.Exit(0)
	}

	for i := 0; i < *count; i++ {
		fmt.Println(generateName())
	}
}

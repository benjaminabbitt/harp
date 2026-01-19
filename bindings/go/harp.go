// Package harp generates random names from adjectives and nouns.
// This is a pure Go binding to the harp-wasm-core Rust library via WebAssembly.
package harp

import (
	"context"
	_ "embed"
	"sync"

	"github.com/tetratelabs/wazero"
	"github.com/tetratelabs/wazero/api"
	"github.com/tetratelabs/wazero/imports/wasi_snapshot_preview1"
)

//go:embed harp_wasm_core.wasm
var wasmBytes []byte

var (
	runtime  wazero.Runtime
	instance api.Module
	initMu   sync.Mutex
	callMu   sync.Mutex
	initErr  error
	initDone bool
)

func initWasm() error {
	initMu.Lock()
	defer initMu.Unlock()

	if initDone {
		return initErr
	}
	initDone = true

	ctx := context.Background()
	runtime = wazero.NewRuntime(ctx)
	wasi_snapshot_preview1.MustInstantiate(ctx, runtime)

	mod, err := runtime.CompileModule(ctx, wasmBytes)
	if err != nil {
		initErr = err
		return err
	}

	instance, err = runtime.InstantiateModule(ctx, mod, wazero.NewModuleConfig())
	if err != nil {
		initErr = err
		return err
	}

	return nil
}

// Options for generating names.
type Options struct {
	// Number of components (2-16). Default: 3
	Components uint8
	// Maximum length per element. 0 means no limit.
	MaxElementLength uint
	// Separator between components. Default: "-"
	Separator string
}

// DefaultOptions returns the default options.
func DefaultOptions() Options {
	return Options{
		Components:       3,
		MaxElementLength: 0,
		Separator:        "-",
	}
}

// GenerateName generates a random name from two adjectives and a noun.
func GenerateName() string {
	return GenerateNameWithOptions(DefaultOptions())
}

// GenerateNameWithOptions generates a random name with custom options.
func GenerateNameWithOptions(opts Options) string {
	if err := initWasm(); err != nil {
		panic("harp: failed to init wasm: " + err.Error())
	}

	// Serialize access to the WASM instance
	callMu.Lock()
	defer callMu.Unlock()

	ctx := context.Background()

	// Get exported functions from persistent instance
	allocFn := instance.ExportedFunction("harp_alloc")
	freeFn := instance.ExportedFunction("harp_free")
	genFn := instance.ExportedFunction("harp_generate_name_with_options")

	// Allocate buffer for separator
	sep := opts.Separator
	if sep == "" {
		sep = "-"
	}
	sepBytes := []byte(sep)

	// Allocate memory for separator
	results, err := allocFn.Call(ctx, uint64(len(sepBytes)))
	if err != nil {
		panic("harp: alloc failed: " + err.Error())
	}
	sepPtr := uint32(results[0])
	if sepPtr != 0 {
		instance.Memory().Write(sepPtr, sepBytes)
	}

	// Allocate a buffer large enough for any name (max ~256 bytes)
	const bufSize = 256
	results, err = allocFn.Call(ctx, bufSize)
	if err != nil {
		panic("harp: alloc failed: " + err.Error())
	}
	bufPtr := uint32(results[0])

	// Generate name into buffer
	results, err = genFn.Call(ctx,
		uint64(opts.Components),
		uint64(opts.MaxElementLength),
		uint64(sepPtr),
		uint64(len(sepBytes)),
		uint64(bufPtr),
		bufSize,
	)
	if err != nil {
		panic("harp: generate failed: " + err.Error())
	}
	actualLen := uint32(results[0])

	// Read result from memory
	buf, ok := instance.Memory().Read(bufPtr, actualLen)
	if !ok {
		panic("harp: failed to read result from memory")
	}
	result := string(buf)

	// Free allocated memory
	freeFn.Call(ctx, uint64(bufPtr), bufSize)
	if sepPtr != 0 {
		freeFn.Call(ctx, uint64(sepPtr), uint64(len(sepBytes)))
	}

	return result
}

// Version returns the library version.
func Version() string {
	if err := initWasm(); err != nil {
		panic("harp: failed to init wasm: " + err.Error())
	}

	callMu.Lock()
	defer callMu.Unlock()

	ctx := context.Background()

	versionFn := instance.ExportedFunction("harp_version")
	versionLenFn := instance.ExportedFunction("harp_version_len")

	results, err := versionLenFn.Call(ctx)
	if err != nil {
		panic("harp: version_len failed: " + err.Error())
	}
	vLen := uint32(results[0])

	results, err = versionFn.Call(ctx)
	if err != nil {
		panic("harp: version failed: " + err.Error())
	}
	vPtr := uint32(results[0])

	buf, ok := instance.Memory().Read(vPtr, vLen)
	if !ok {
		panic("harp: failed to read version from memory")
	}

	return string(buf)
}

package build

import (
	"fmt"
	"log"
	"os"
	"os/exec"
	"strings"
)

type BuildMode string

const (
	BuildModeNone    BuildMode = ""
	BuildModeDebug   BuildMode = "debug"
	BuildModeRelease BuildMode = "release"
)

type Compiler string

const (
	CompilerDefault   Compiler = ""
	CompilerLLVM      Compiler = "llvm"
	CompilerCranelift Compiler = "cranelift"
)

type Config struct {
	Rebuild  bool
	Mode     BuildMode
	Compiler Compiler
}

func ParseBuildMode(s string) (BuildMode, error) {
	switch strings.ToLower(s) {
	case "", "none":
		return BuildModeNone, nil
	case "debug", "dev", "d":
		return BuildModeDebug, nil
	case "release", "rel", "r":
		return BuildModeRelease, nil
	default:
		return BuildModeNone, fmt.Errorf("invalid build mode: %s (valid: debug, release)", s)
	}
}

func ParseCompiler(s string) (Compiler, error) {
	switch strings.ToLower(s) {
	case "", "default", "llvm", "ll", "l":
		return CompilerLLVM, nil
	case "cranelift", "cl", "c", "fast":
		return CompilerCranelift, nil
	default:
		return CompilerDefault, fmt.Errorf("invalid compiler: %s (valid: llvm, cranelift)", s)
	}
}

func RebuildEngine(config Config) error {
	if !config.Rebuild {
		return nil
	}

	mode := config.Mode
	if mode == BuildModeNone {
		mode = BuildModeRelease
	}

	compiler := config.Compiler
	if compiler == CompilerDefault {
		compiler = CompilerLLVM
	}

	log.Printf("Rebuilding engine (mode: %s, compiler: %s)...", mode, compiler)

	args := buildCargoArgs(mode, compiler)
	env := buildEnv(mode, compiler)

	cmd := exec.Command(args[0], args[1:]...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Env = append(os.Environ(), env...)

	if err := cmd.Run(); err != nil {
		return fmt.Errorf("build failed: %w", err)
	}

	log.Println("Build completed successfully")
	return nil
}

func buildCargoArgs(mode BuildMode, compiler Compiler) []string {
	args := []string{"cargo"}

	if compiler == CompilerCranelift {
		args = []string{"cargo", "+nightly"}
	}

	args = append(args, "build", "-p", "yavashark_test262")

	if mode == BuildModeRelease {
		args = append(args, "--release")
	}

	if compiler == CompilerCranelift {
		args = append(args, "-Zcodegen-backend")
	}

	return args
}

func buildEnv(mode BuildMode, compiler Compiler) []string {
	var env []string

	if compiler == CompilerCranelift {
		env = append(env, "RUSTFLAGS=-Zthreads=32")

		if mode == BuildModeRelease {
			env = append(env, "CARGO_PROFILE_RELEASE_CODEGEN_BACKEND=cranelift")
		} else {
			env = append(env, "CARGO_PROFILE_DEV_CODEGEN_BACKEND=cranelift")
		}
	}

	return env
}

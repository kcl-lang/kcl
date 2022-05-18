// Copyright 2022 The KCL Authors. All rights reserved.

//go:build ingore
// +build ingore

package main

import (
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

var (
	flagOutDir = flag.String("outdir", "./_output/kclvm-windows", "set output dir")
)

func main() {
	flag.Parse()
	if *flagOutDir == "" {
		panic("-outdir missing")
	}

	// copy {root}/libs/rust-libstd-name.txt
	err := os.WriteFile(
		filepath.Join(*flagOutDir, "libs", "rust-libstd-name.txt"),
		[]byte(getRustStdDllName()),
		0666,
	)
	if err != nil {
		panic(err)
	}

	// copy {root}/std-***.dll
	cpFile(
		filepath.Join(*flagOutDir, getRustStdDllName()),
		filepath.Join(getRustWinX64LibDir(), getRustStdDllName()),
	)
	// copy {root}/libs/std-***.dll.lib
	cpFile(
		filepath.Join(*flagOutDir, "libs", getRustStdDllLibName()),
		filepath.Join(getRustWinX64LibDir(), getRustStdDllLibName()),
	)
}

func getRustSysRoot() string {
	// rustc --print sysroot
	out, err := exec.Command("rustc", "--print", "sysroot").Output()
	if err != nil {
		panic(err)
	}
	return strings.TrimSpace(string(out))
}

func getRustWinX64LibDir() string {
	return filepath.Join(
		getRustSysRoot(), "lib", "rustlib", "x86_64-pc-windows-msvc", "lib",
	)
}

func getRustStdDllName() string {
	matches, err := filepath.Glob(getRustWinX64LibDir() + "/std-*.dll.lib")
	if err != nil {
		panic(err)
	}
	if len(matches) != 1 {
		panic(fmt.Sprintf("glob(\"%s/std-*.dll.lib\") failed", getRustWinX64LibDir()))
	}
	dllLib := filepath.Base(matches[0])
	return strings.TrimSuffix(dllLib, ".lib")
}

func getRustStdDllLibName() string {
	return getRustStdDllName() + ".lib"
}

func cpFile(dst, src string) {
	err := os.MkdirAll(filepath.Dir(dst), 0777)
	if err != nil && !os.IsExist(err) {
		log.Fatal("cpFile: ", err)
	}
	fsrc, err := os.Open(src)
	if err != nil {
		log.Fatal("cpFile: ", err)
	}
	defer fsrc.Close()

	fdst, err := os.Create(dst)
	if err != nil {
		log.Fatal("cpFile: ", err)
	}
	defer fdst.Close()
	if _, err = io.Copy(fdst, fsrc); err != nil {
		log.Fatal("cpFile: ", err)
	}
}

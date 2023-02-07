// Copyright 2021 The KCL Authors. All rights reserved.

//go:build ingore && windows
// +build ingore,windows

package main

import (
	"os"
	"os/exec"
	"path/filepath"
)

func main() {

	kclvm_install_dir, _ := filepath.Abs(filepath.Dir(filepath.Dir(os.Args[0])))

	os.Setenv("KCLVM_CLI_BIN_PATH", filepath.Join(kclvm_install_dir, "bin"))
	os.Setenv("PYTHONPATH", filepath.Join(kclvm_install_dir, "lib", "site-packages"))

	cmd := exec.Command("python3", os.Args[1:]...)
	cmd.Env = os.Environ()
	cmd.Stderr = os.Stderr
	cmd.Stdout = os.Stdout
	cmd.Stdin = os.Stdin
	if err := cmd.Start(); err != nil {
		os.Exit(1)
	}
}

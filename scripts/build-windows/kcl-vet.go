// Copyright 2023 The KCL Authors. All rights reserved.

//go:build ingore && windows
// +build ingore,windows

package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

func main() {
	var args []string
	args = append(args, "/C", "kclvm-cli", "vet")
	args = append(args, os.Args[1:]...)

	os.Exit(KclvmCliMain(args))
}

func KclvmCliMain(args []string) int {
	inputPath, err := os.Executable()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Input path does not exist")
		os.Exit(1)
	}
	kclvm_install_dir := filepath.Dir(filepath.Dir(inputPath))

	cmd := exec.Command("cmd", args...)
	cmd.Stderr = os.Stderr
	cmd.Stdout = os.Stdout

	SetEnv(kclvm_install_dir, cmd)

	err = cmd.Run()
	if err != nil {
		fmt.Fprintln(os.Stderr, "exec failed:", err)
		os.Exit(1)
	}
	return 0
}

func SetEnv(kclvm_install_dir string, cmd *exec.Cmd) {
	bin_path := filepath.Join(kclvm_install_dir, "bin")
	site_packages_path := filepath.Join(kclvm_install_dir, "lib", "site-packages")

	os.Setenv("PATH", os.Getenv("PATH")+";"+bin_path)
	cmd.Env = os.Environ()
	cmd.Env = append(cmd.Env, "KCLVM_CLI_BIN_PATH="+bin_path)
	cmd.Env = append(cmd.Env, "PYTHONPATH="+site_packages_path)
}

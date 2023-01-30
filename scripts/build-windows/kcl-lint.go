// Copyright 2021 The KCL Authors. All rights reserved.

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
	// python3 -m kclvm ...

	var args []string
	args = append(args, "/C", "python3", "-m", "kclvm.tools.lint.lint")
	args = append(args, os.Args[1:]...)

	os.Exit(Py_Main(args))
}

func Py_Main(args []string) int {
	inputPath, err := os.Executable()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Input path does not exist")
		os.Exit(1)
	}
	kclvm_install_dir_bin := filepath.Dir(inputPath)
	Install_Kclvm(kclvm_install_dir_bin)
	kclvm_install_dir := filepath.Dir(kclvm_install_dir_bin)

	cmd := exec.Command("cmd", args...)
	cmd.Stderr = os.Stderr
	cmd.Stdout = os.Stdout

	Set_Env(kclvm_install_dir, cmd)

	err = cmd.Run()
	if err != nil {
		fmt.Fprintln(os.Stderr, "exec failed:", err)
		os.Exit(1)
	}
	return 0
}

func Install_Kclvm(installed_path string) {
	// Check if Python3 is installed
	cmd := exec.Command("cmd", "/C", "where python3")
	cmd.Stderr = os.Stderr

	err := cmd.Run()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Python3 is not installed, details: ", err)
		os.Exit(1)
	}

	// Check if "installed" file exists
	outputPath := filepath.Join(installed_path, "kclvm_installed")
	if _, err := os.Stat(outputPath); err == nil {
		return
	}

	// Install kclvm module using pip
	cmd = exec.Command("cmd", "/C", "python3", "-m", "pip", "install", "--use-pep517", "kclvm")
	cmd.Stderr = os.Stderr

	err = cmd.Run()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Pip install kclvm falied ", err)
		os.Exit(1)
	}

	// Create "installed" file
	f, err := os.Create(outputPath)
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error creating file: ", err)
		os.Exit(1)
	}
	defer f.Close()
}

func Set_Env(kclvm_install_dir string, cmd *exec.Cmd) {
	bin_path := filepath.Join(kclvm_install_dir, "bin")
	site_packages_path := filepath.Join(kclvm_install_dir, "lib", "site-packages")

	os.Setenv("PATH", os.Getenv("PATH")+";"+bin_path)
	cmd.Env = os.Environ()
	cmd.Env = append(cmd.Env, "KCLVM_CLI_BIN_PATH="+bin_path)
	cmd.Env = append(cmd.Env, "PYTHONPATH="+site_packages_path)
}

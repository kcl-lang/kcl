// Copyright 2021 The KCL Authors. All rights reserved.

//go:build ingore && windows
// +build ingore,windows

package main

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
)

func main() {
	// python3 -m kclvm ...

	var args []string
	args = append(args, "/C", "python3", "-m", "kclvm.tools.plugin")
	args = append(args, os.Args[1:]...)

	os.Exit(Py_Main(args))
}

func Py_Main(args []string) int {
	inputPath, err := os.Executable()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Input path does not exist")
		os.Exit(1)
	}
	Install_Kclvm()
	kclvm_install_dir := filepath.Dir(filepath.Dir(inputPath))

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

func Install_Kclvm() {
	// Check if Python3 is installed
	cmd := exec.Command("cmd", "/C", "where python3")
	cmd.Stderr = os.Stderr

	err := cmd.Run()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Python3 is not installed, details: ", err)
		os.Exit(1)
	}

	cmd = exec.Command("cmd", "/C", "python3", "-c", "\"import pkgutil;print(bool(pkgutil.find_loader('kclvm')))\"")
	var out bytes.Buffer
	cmd.Stdout = &out
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		fmt.Fprintln(os.Stderr, "check python3 kclvm failed: ", err)
		os.Exit(1)
	}

	is_installed, err := strconv.ParseBool(out.String())

	// Check if kclvm has been installed.
	if err == nil && is_installed {
		return
	}

	// Install kclvm module using pip
	cmd = exec.Command("cmd", "/C", "python3", "-m", "pip", "install", "-U", "kclvm", "--user")
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		fmt.Fprintln(os.Stderr, "Pip install kclvm falied ", err)
		os.Exit(1)
	}
}

func Set_Env(kclvm_install_dir string, cmd *exec.Cmd) {
	bin_path := filepath.Join(kclvm_install_dir, "bin")
	site_packages_path := filepath.Join(kclvm_install_dir, "lib", "site-packages")

	os.Setenv("PATH", os.Getenv("PATH")+";"+bin_path)
	cmd.Env = os.Environ()
	cmd.Env = append(cmd.Env, "KCLVM_CLI_BIN_PATH="+bin_path)
	cmd.Env = append(cmd.Env, "PYTHONPATH="+site_packages_path)
}

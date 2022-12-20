// Copyright 2021 The KCL Authors. All rights reserved.

//go:build ingore && windows
// +build ingore,windows

package main

import (
	"os"
	"path/filepath"
	"syscall"
	"unsafe"
)

func main() {
	// kclvm -m kclvm ...
	var args []string
	args = append(args, os.Args[0])
	args = append(args, "-m", "kclvm.tools.validation")
	args = append(args, os.Args[1:]...)
	os.Exit(Py_Main(args))
}

var (
	python39_dll = syscall.NewLazyDLL(findKclvm_dllPath())
	proc_Py_Main = python39_dll.NewProc("Py_Main")
)

// int Py_Main(int argc, wchar_t **argv)
func Py_Main(args []string) int {
	c_args := make([]*uint16, len(args)+1)
	for i, s := range args {
		c_args[i] = syscall.StringToUTF16Ptr(s)
	}
	ret, _, _ := proc_Py_Main.Call(uintptr(len(args)), uintptr(unsafe.Pointer(&c_args[0])))
	return int(ret)
}

func findKclvm_dllPath() string {
	kclvmName := "python39.dll"

	if exePath, _ := os.Executable(); exePath != "" {
		exeDir := filepath.Dir(exePath)
		if fi, _ := os.Stat(filepath.Join(exeDir, kclvmName)); fi != nil && !fi.IsDir() {
			return filepath.Join(exeDir, kclvmName)
		}
	}
	if wd, _ := os.Getwd(); wd != "" {
		if fi, _ := os.Stat(filepath.Join(wd, kclvmName)); fi != nil && !fi.IsDir() {
			return filepath.Join(wd, kclvmName)
		}
		wd = filepath.Join(wd, "_output/kclvm-windows")
		if fi, _ := os.Stat(filepath.Join(wd, kclvmName)); fi != nil && !fi.IsDir() {
			return filepath.Join(wd, kclvmName)
		}
	}

	return kclvmName
}

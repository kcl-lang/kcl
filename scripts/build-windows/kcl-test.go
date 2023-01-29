package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
)

func main() {
	pwd_path, _ := os.Getwd()
	kcl_go_path := filepath.Join(pwd_path, "kcl-go")
	if _, err := os.Stat(kcl_go_path); os.IsNotExist(err) {
		fmt.Println("kcl-go not found, please check the installation")
		os.Exit(1)
	}
	os.Setenv("PYTHONPATH", "")
	cmd := exec.Command(kcl_go_path, "test")
	cmd.Args = append(cmd.Args, os.Args[1:]...)

	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Run()
}

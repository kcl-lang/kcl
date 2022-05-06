// Copyright 2021 The KCL Authors. All rights reserved.

//go:build ignore
// +build ignore

package main

import (
	"fmt"
	"time"
)

func main() {
	t := time.Now()
	fmt.Printf(
		"%04d%02d%02d-%02d%02d%02d",
		t.Year(), t.Month(), t.Day(),
		t.Hour(), t.Minute(), t.Second(),
	)
}

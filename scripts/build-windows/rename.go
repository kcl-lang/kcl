// Copyright 2021 The KCL Authors. All rights reserved.

//go:build ingore
// +build ingore

package main

import (
	"flag"
	"log"
	"os"
)

var (
	flagOldFile = flag.String("old", "", "set old file")
	flagNewFile = flag.String("new", "", "set new file")
)

func main() {
	flag.Parse()
	if *flagOldFile == "" || *flagNewFile == "" {
		flag.Usage()
		os.Exit(1)
	}
	err := os.Rename(*flagOldFile, *flagNewFile)
	if err != nil {
		log.Fatal(err)
	}
}

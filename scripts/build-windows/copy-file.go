// Copyright 2021 The KCL Authors. All rights reserved.

//go:build ingore
// +build ingore

package main

import (
	"flag"
	"io"
	"log"
	"os"
	"path/filepath"
)

var (
	flagDst = flag.String("dst", "", "set dst path")
	flagSrc = flag.String("src", "", "set src path")
)

func init() {
	log.SetFlags(log.Lshortfile)
}

func main() {
	flag.Parse()

	if *flagDst == "" {
		log.Fatal("dst path missing")
	}
	if *flagSrc == "" {
		log.Fatal("src path missing")
	}

	cpFile(*flagDst, *flagSrc)
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

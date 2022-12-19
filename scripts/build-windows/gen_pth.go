// Copyright 2021 The KCL Authors. All rights reserved.

//go:build ingore
// +build ingore

package main

import (
	"flag"
	"os"
)

var (
	flagFile = flag.String("file", "./_output/python39._pth", "set output file")
)

func main() {
	flag.Parse()
	if err := os.WriteFile(*flagFile, []byte(code), 0666); err != nil {
		panic(err)
	}
}

const code = `
python39.zip
Lib
Lib\site-packages
.

# Uncomment to run site.main() automatically
#import site
`

package main

import (
	"flag"
)

func main() {
	flag.Parse()
	args := flag.Args()
	CLI(args...)
}

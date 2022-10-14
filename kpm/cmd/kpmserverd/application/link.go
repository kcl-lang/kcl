package application

import _ "unsafe"

//go:linkname FastRand runtime.fastrand
func FastRand() uint32

//go:linkname Nanotime1 runtime.nanotime1
func Nanotime1() int64

////go:linkname Walltime runtime.walltime
//func Walltime() int64

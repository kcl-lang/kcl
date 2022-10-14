package main

import (
	"encoding/hex"
	"github.com/cespare/xxhash/v2"
)

func EncodeToString(src [64]byte) string {
	dst := make([]byte, hex.EncodedLen(len(src)))
	Encode(dst, src)
	return string(dst)
}
func Encode(dst []byte, src [64]byte) int {
	j := 0
	for _, v := range src {
		dst[j] = hextable[v>>4]
		dst[j+1] = hextable[v&0x0f]
		j += 2
	}
	return len(src) * 2
}
func HashMod(b []byte) string {
	t := xxhash.Sum64(b) % 256
	t1 := t % 16
	t2 := t / 16
	var c [2]byte
	c[0] = hextable[t2]
	c[1] = hextable[t1]
	return string(c[:])
}

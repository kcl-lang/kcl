package main

import "sync"

type Set map[string]struct{}

func NewSet() Set {

	return make(map[string]struct{}, 64)
}
func (s Set) SAdd(strs ...string) {
	for i := 0; i < len(strs); i++ {
		s[strs[i]] = struct{}{}
	}
}
func (s Set) SMembers() []string {
	strs := make([]string, len(s))
	strs = strs[:0]
	for s2 := range s {
		strs = append(strs, s2)
	}
	return strs
}
func (s Set) SIsMember(str string) bool {
	_, err := s[str]
	return err
}
func (s Set) SRem(str string) error {
	_, err := s[str]
	if err {

	}
	delete(s, str)
	return nil
}

// SUnion 并集
func (s Set) SUnion(sets ...Set) Set {
	newset := AcquireSet()
	for i := 0; i < len(sets); i++ {
		for key, _ := range sets[i] {
			newset.SAdd(key)
		}
	}
	return newset
}
func (s Set) SCard() int {
	return len(s)
}

func (s Set) Reset() {
	for s2, _ := range s {
		delete(s, s2)
	}
}

var pool = sync.Pool{
	New: func() any {
		return Set{}
	},
}

func AcquireSet() Set {
	return pool.Get().(Set)
}

func ReleaseSet(s Set) {
	s.Reset()
	pool.Put(s)
}

package main

import (
	"errors"
	"strconv"
	"strings"
)

type Version struct {
	Major int
	Minor int
	Patch int
}

func (v Version) ToString() string {
	return "v" + strconv.Itoa(v.Major) + "." + strconv.Itoa(v.Minor) + "." + strconv.Itoa(v.Patch)
}
func (v Version) NewFromString(str string) error {
	if str[0] == 'v' || str[0] == 'V' {
		str = str[1:]
	}
	vd := strings.Split(str, ".")
	if len(vd) != 3 {
		//处理出错
		return errors.New("faulty data")
	}
	major, err := strconv.Atoi(vd[0])
	if err != nil {
		return err
	}
	v.Major = major
	minor, err := strconv.Atoi(vd[0])
	if err != nil {
		return err
	}
	v.Minor = minor
	patch, err := strconv.Atoi(vd[0])
	if err != nil {
		return err
	}
	v.Patch = patch
	return nil
}
func (v Version) Cmp(nv Version) int {
	if v.Major != nv.Major {
		if v.Major > nv.Major {
			return 1
		} else {
			return -1
		}
	}
	if v.Minor != nv.Minor {
		if v.Minor > nv.Minor {
			return 1
		} else {
			return -1
		}
	}
	if v.Patch != nv.Patch {
		if v.Patch > nv.Patch {
			return 1
		} else {
			return -1
		}
	}
	return 0
}

package main

import "path/filepath"

const hextable = "0123456789abcdef"
const debug = false
const ExternalDependencies = "external"
const Separator = string(filepath.Separator)

const (
	StdOkResp        = `{"code":0,"msg":"ok"}`
	StdErrResp       = `{"code":1,"msg":"err"}`
	StdArgsWrongResp = `{"code":2,"msg":"ArgsWrong"}`
)

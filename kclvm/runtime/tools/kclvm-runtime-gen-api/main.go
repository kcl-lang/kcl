// Copyright The KCL Authors. All rights reserved.

package main

import (
	"bytes"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"text/template"
)

var (
	flagRoot           = flag.String("root", ".", "set kclvm-runtime root")
	flagGenCApi        = flag.String("c-api", "", "set c header file")
	flagGenLLApi       = flag.String("ll-api", "", "set llvm-ll file")
	flagGenRustApiEnum = flag.String("rust-api-enum", "", "set rust-api-enum file")
	flagGenRustApiAddr = flag.String("rust-api-addr", "", "set rust-api-addr file")
)

func main() {
	flag.Parse()

	specList := LoadAllApiSpec(*flagRoot)
	if filename := *flagGenCApi; filename != "" {
		src := genCApi(specList)
		if err := os.WriteFile(filename, []byte(src), 0666); err != nil {
			panic(err)
		}
	}
	if filename := *flagGenLLApi; filename != "" {
		src := genLLApi(specList)
		if err := os.WriteFile(filename, []byte(src), 0666); err != nil {
			panic(err)
		}
	}
	if filename := *flagGenRustApiEnum; filename != "" {
		src := genRustApiEnum(specList)
		if err := os.WriteFile(filename, []byte(src), 0666); err != nil {
			panic(err)
		}
	}
	if filename := *flagGenRustApiAddr; filename != "" {
		src := genRustApiAddr(specList)
		if err := os.WriteFile(filename, []byte(src), 0666); err != nil {
			panic(err)
		}
	}
}

func genCApi(specs []ApiSpec) string {
	tmpl, err := template.New("c-api").Parse(tmplCApi)
	if err != nil {
		panic(err)
	}

	var buf bytes.Buffer
	err = tmpl.Execute(&buf, specs)
	if err != nil {
		panic(err)
	}
	return fmtCode(buf.String())
}

func genLLApi(specs []ApiSpec) string {
	tmpl, err := template.New("ll-api").Parse(tmplLLApi)
	if err != nil {
		panic(err)
	}

	var buf bytes.Buffer
	err = tmpl.Execute(&buf, specs)
	if err != nil {
		panic(err)
	}
	return fmtCode(buf.String())
}

func genRustApiEnum(specs []ApiSpec) string {
	tmpl, err := template.New("rust-api-enum").Parse(tmplRustEnum)
	if err != nil {
		panic(err)
	}

	var buf bytes.Buffer
	err = tmpl.Execute(&buf, specs)
	if err != nil {
		panic(err)
	}
	return fmtCode(buf.String())
}

func genRustApiAddr(specs []ApiSpec) string {
	tmpl, err := template.New("rust-api-addr").Parse(tmplRustAddr)
	if err != nil {
		panic(err)
	}

	var buf bytes.Buffer
	err = tmpl.Execute(&buf, specs)
	if err != nil {
		panic(err)
	}
	return fmtCode(buf.String())
}

func fmtCode(s string) string {
	for {
		if !strings.Contains(s, "\n\n\n") {
			s = strings.TrimSpace(s) + "\n"
			return s
		}
		s = strings.ReplaceAll(s, "\n\n\n", "\n\n")
	}
}

// ----------------------------------------------------------------------------

const (
	apiSpecPrefix_Name  = "// api-spec:"
	apiSpecPrefix_CApi  = "// api-spec(c):"
	apiSpecPrefix_LLApi = "// api-spec(llvm):"
)

type ApiSpec struct {
	File   string
	Line   int
	Name   string // api-spec:       kclvm_context_new
	SpecC  string // api-spec(c):    i32 kclvm_context_new();
	SpecLL string // api-spec(llvm): declare i32* @kclvm_context_new()
	IsType bool
}

func LoadAllApiSpec(root string) []ApiSpec {
	m := make(map[string]ApiSpec)
	filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			panic(err)
		}
		if info.IsDir() {
			return nil
		}

		if !strings.HasSuffix(path, ".rs") {
			return nil
		}

		data, err := os.ReadFile(path)
		if err != nil {
			panic(err)
		}

		var spec ApiSpec
		for i, line := range strings.Split(string(data), "\n") {
			line := strings.TrimSpace(line)
			switch {
			case strings.HasPrefix(line, apiSpecPrefix_Name):
				spec.File = path
				spec.Line = i + 1
				spec.Name = strings.TrimSpace(strings.TrimPrefix(line, apiSpecPrefix_Name))
				spec.IsType = strings.HasSuffix(spec.Name, "_t")
			case strings.HasPrefix(line, apiSpecPrefix_CApi):
				if spec.SpecC != "" {
					spec.SpecC += " "
				}
				spec.SpecC += strings.TrimSpace(strings.TrimPrefix(line, apiSpecPrefix_CApi))
			case strings.HasPrefix(line, apiSpecPrefix_LLApi):
				if spec.SpecLL != "" {
					spec.SpecLL += " "
				}
				spec.SpecLL += strings.TrimSpace(strings.TrimPrefix(line, apiSpecPrefix_LLApi))
			default:
				if matched, _ := regexp.MatchString(`//\s*api-spec`, line); matched {
					panic(fmt.Errorf("%s:%d invalid 'api-spec'", path, i+1))
				}
				if spec.Name != "" {
					if x, ok := m[spec.Name]; ok {
						fmt.Printf("WARN: %s:%d %s api-spec exits (%s:%d)\n", path, i+1, spec.Name, x.File, x.Line)
					}
					m[spec.Name] = spec
				}
				spec = ApiSpec{}
			}
		}

		return nil
	})

	var specs []ApiSpec
	for _, x := range m {
		specs = append(specs, x)
	}
	sort.Slice(specs, func(i, j int) bool {
		return specs[i].Name < specs[j].Name
	})
	return specs
}

// ----------------------------------------------------------------------------

const tmplCApi = `
{{$specList := .}}

// Copyright The KCL Authors. All rights reserved.

// Auto generated, DONOT EDIT!!!

#pragma once

#ifndef _kclvm_h_
#define _kclvm_h_

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// please keep same as 'kclvm/runtime/src/kind/mod.rs#Kind'

enum kclvm_kind_t {
    Invalid = 0,

    // only for value

    Undefined = 1, 
    None = 2,

    // for value & type

    Bool = 3,
    Int = 4,
    Float = 5,
    Str = 6,
    List = 7,
    Dict = 8,

    Schema = 9,
    Error = 10,

    // only for type

    Any = 11,
    Union = 12,

    BoolLit = 13,
    IntLit = 14,
    FloatLit = 15,
    StrLit = 16,

    Func = 17,

    // max num

    Max = 18,
};

{{range $_, $spec := $specList}}
{{if ($spec.IsType)}}{{$spec.SpecC}}{{end}}
{{end}}

{{range $_, $spec := $specList}}
{{if (not $spec.IsType)}}{{$spec.SpecC}}{{end}}
{{end}}

#ifdef __cplusplus
} // extern "C"
#endif

#endif // _kclvm_h_
`

// ----------------------------------------------------------------------------

const tmplLLApi = `
{{$specList := .}}

; Copyright The KCL Authors. All rights reserved.

; Auto generated, DONOT EDIT!!!

{{range $_, $spec := $specList}}
{{if ($spec.IsType)}}{{$spec.SpecLL}}{{end}}
{{end}}

{{range $_, $spec := $specList}}
{{if (not $spec.IsType)}}{{$spec.SpecLL}}{{end}}
{{end}}

define void @__kcl_keep_link_runtime(%kclvm_value_ref_t* %_a, %kclvm_context_t* %_b) {
	call %kclvm_value_ref_t* @kclvm_value_None(%kclvm_context_t* %_b)
	ret void
}
`

// ----------------------------------------------------------------------------

const tmplRustEnum = `
{{$specList := .}}

// Copyright The KCL Authors. All rights reserved.

// Auto generated, DONOT EDIT!!!

#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum ApiType {
    Value,
}

impl std::fmt::Display for ApiType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ApiType::Value => write!(f, "{:?}", "api::kclvm::Value"),
        }
    }
}

impl ApiType {
    #[allow(dead_code)]
    pub fn name(&self) -> String {
        format!("{self:?}")
    }
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum ApiFunc {
	{{range $_, $spec := $specList}}{{if (not $spec.IsType)}}
	{{- $spec.Name}},
	{{end}}{{end}}
}

impl std::fmt::Display for ApiFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ApiFunc {
    #[allow(dead_code)]
    pub fn name(&self) -> String {
        format!("{self:?}")
    }
}
`

// ----------------------------------------------------------------------------

const tmplRustAddr = `
{{$specList := .}}

// Copyright The KCL Authors. All rights reserved.

// Auto generated, DONOT EDIT!!!

#[allow(dead_code)]
pub fn _kclvm_get_fn_ptr_by_name(name: &str) -> u64 {
	match name {
		{{- range $_, $spec := $specList -}}{{if (not $spec.IsType)}}
		"{{$spec.Name}}" => crate::{{$spec.Name}} as *const () as u64,
		{{- end}}{{end}}
		_ => panic!("unknown {name}"),
	}
}
`

// ----------------------------------------------------------------------------

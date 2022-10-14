package main

import "path/filepath"

const hextable = "0123456789abcdef"
const debuglog = false
const ExternalDependencies = "external"
const InternalDependencies = "internal"

const Separator = string(filepath.Separator)
const DefaultKclModContent = `[expected]
kclvm_version=`

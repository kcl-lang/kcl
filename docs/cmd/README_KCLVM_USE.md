# Kusion Configuration Language Virtual Machine (KCLVM) Command-Line Interface (CLI) Tool

KCLVM mainly uses a CLI tool to help us to build our cloud native configuration application, and the CLI tool is a cross-platform tool chain for compiling the KCL file. We can easily use it to generate our wanted configuration YAML file.

## Abstract

```cli
kcl [-h] [-D ARGUMENT] [-S PATH_SELECTOR] [-O OVERRIDES]
    [-Y [SETTING [SETTING ...]]] [-o OUTPUT] [-n] [-r] [-c] [-s] [-v]
    [-d] [-p] [-L] [-l] [-V] [--target {native,wasm}]
    [file [file ...]]
```

## Parameters

* `ARGUMENT`: The top-level argument.
* `OUTPUT`: The output file.
* `SETTING`: The top-level YAML setting file.
* `PATH_SELECTOR`: The configuration selector path.
* `OVERRIDES`: The configuration override path and value.

## Arguments

### Positional Arguments

* `file`: The input KCL files to be compiled.

### Optional Arguments

* `-h|--help`: Show the help message and exit.
* `-D|--argument`: Specify the top-level argument.
* `-Y|--setting`: Specify the top-level setting file.
* `-o|--output`: Specify the output file.
* `-n|--disable-none`: Disable dumping None values.
* `-r|--strict-range-check`: Do perform strict numeric range check.
* `-c|--compile-only`: Compile only and do not generate the YAML file.
* `-s|--save-temps`: Save intermediate files.
* `-v|--verbose`: Run in verbose mode.
* `-d|--debug`: Run in debug mode (for developers only).
* `-p|--profile`: Perform profiling.
* `-L|--list-attributes`: Show schema attributes list.
* `-l|--list-options`: Show kcl options list.
* `-V|--version`: Show the kcl version.
* `-S|--path-selector`: Specify the path selector.
* `-O|--overrides`: Specify the configuration override path and value.
* `--target {native,wasm}`: Specify the target type

## Examples

* If we have written our KCL files, KCL can be invoked as:

```
kcl your_config.k
```

* In addition, we can specify the location of the output:

```
kcl your_config.k -o your_yaml.yaml
```

* We can use `-D ARGUMENT` or `--argument ARGUMENT` to specify the top-level arguments:

```
kcl your_config.k -D your_arguments

Examples:
kcl your_config.k -D argsName=argsValue
```

* We can use `-Y SETTING` or `--setting SETTING` to specify the top-level arguments through the YAML file:

```
kcl your_config.k -Y your_setting_file.yaml
```

* If we don’t want to display `none` values in the generated file, we can use the parameter `-n`:

```
kcl your_config.k -n
```

* If we want to perform the strict numeric range check, we can use the parameter `-r`:

```
kcl your_config.k -r
```

* If we want to save intermediate files, we can use the parameter `-s`:

```
kcl your_config.k -s
```

* If we want to show schema attributes list and kcl options list, we can use the parameter `-L` and `-l`:

```
kcl your_config.k -L -l
```

* If we want to get part of KCL configuration, we can use `-S` or `--path-selector` to specify the configuration override path and value:

```
kcl your_config.k -S pkg:variable_path
```

* If we want to override part of the KCL configuration content, we can use `-O` or `--overrides` to specify the configuration override path and value:

```
kcl your_config.k -O pkg:variable_path=value
```

* If we want to compile KCL code into a native dynamic link library, we can use `--target` to specify the `native` target.

```
kcl your_config.k --target native
```

* If we want to compile KCL code into a WASM module, we can use `--target` to specify the `wasm` target.

```
kcl your_config.k --target wasm
```

* For more information, we can use the following command to show the help message:

```
kcl --help
```

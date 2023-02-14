cd %~dp0

call .\\scripts\\build-windows\\build.bat

set "bin_path=%cd%\scripts\build-windows\_output\kclvm-windows\bin"
set "path=%path%;%bin_path%"

@REM rust unit test
cd .\\kclvm
cargo test -p kclvm-*
cd %~dp0

@REM rust runtime test
cd .\\kclvm\\tests\\test_units
kclvm -m pytest -vv
cd %~dp0

@REM konfig test
call .\\test\\integration\\test_konfig.bat

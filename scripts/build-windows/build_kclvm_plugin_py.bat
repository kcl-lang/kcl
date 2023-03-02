:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: install kclvm-plugin python module
cd ..\..\kclvm\plugin
python3 setup.py install_lib --install-dir=..\..\scripts\build-windows\_output\kclvm-windows\lib\site-packages 

cd %~dp0
go run .\copy-dir.go ..\..\plugins ..\..\scripts\build-windows\_output\kclvm-windows\plugins
cd %~dp0

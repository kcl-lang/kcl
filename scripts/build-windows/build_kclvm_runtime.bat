:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal
cd %~dp0

:: Copy KCLVM C API header
copy ..\..\kclvm\runtime\src\_kclvm.h .\_output\kclvm-windows\include\kclvm.h
cd %~dp0

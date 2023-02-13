:: Copyright 2021 The KCL Authors. All rights reserved.

setlocal

cd %~dp0

rmdir _output
del /s *.obj
del /s *.exp
del /s *.lib
del /s *.dll

del *.zip

# Copyright The KCL Authors. All rights reserved.

Set-Location $PSScriptRoot

Remove-Item -Recurse -Force "_output"
Remove-Item -Recurse -Force "*.obj"
Remove-Item -Recurse -Force "*.exp"
Remove-Item -Recurse -Force "*.lib"
Remove-Item -Recurse -Force "*.dll"
Remove-Item -Force "*.zip"

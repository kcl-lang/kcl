# Copyright The KCL Authors. All rights reserved.

Set-Location $PSScriptRoot
# 1. Install kclvm_cli_cdylib.dll
Set-Location "..\..\kclvm"
cargo build --release
Set-Location $PSScriptRoot

New-Item -ErrorAction Ignore -Path ".\_output" -ItemType "directory"
New-Item -ErrorAction Ignore -Path ".\_output\kclvm-windows" -ItemType "directory"
New-Item -ErrorAction Ignore -Path ".\_output\kclvm-windows\bin" -ItemType "directory"
New-Item -ErrorAction Ignore -Path ".\_output\kclvm-windows\include" -ItemType "directory"

Copy-Item -Path "..\..\kclvm\target\release\kclvm_cli_cdylib.dll" -Destination ".\_output\kclvm-windows\bin\kclvm_cli_cdylib.dll" -Force
Copy-Item -Path "..\..\kclvm\target\release\kclvm_cli_cdylib.dll.lib" -Destination ".\_output\kclvm-windows\bin\kclvm_cli_cdylib.lib" -Force
Copy-Item -Path "..\..\kclvm\target\release\kclvm_cli_cdylib.dll.lib" -Destination "..\..\kclvm\target\release\kclvm_cli_cdylib.lib" -Force

Set-Location $PSScriptRoot
# 2. Install kclvm CLI
Set-Location "..\..\cli"
cargo build --release
Set-Location $PSScriptRoot
Copy-Item -Path "..\..\cli\target\release\kclvm_cli.exe" -Destination ".\_output\kclvm-windows\bin\" -Force

Set-Location $PSScriptRoot
# 3. Install kcl language server
Set-Location "..\..\kclvm\tools\src\LSP"
cargo build --release
Set-Location $PSScriptRoot
Copy-Item -Path "..\..\kclvm\target\release\kcl-language-server.exe" -Destination ".\_output\kclvm-windows\bin\"

Set-Location $PSScriptRoot
# 4. Copy KCLVM C API header
Copy-Item -Path "..\..\kclvm\runtime\src\_kclvm.h" -Destination ".\_output\kclvm-windows\include\kclvm.h" -Force

Set-Location $PSScriptRoot
# Install hello.k
Copy-Item -Path "..\..\samples\hello.k" -Destination ".\_output\kclvm-windows" -Force

# Run KCL files
.\_output\kclvm-windows\bin\kclvm_cli.exe run ..\..\samples\fib.k
.\_output\kclvm-windows\bin\kclvm_cli.exe run ..\..\samples\hello.k
.\_output\kclvm-windows\bin\kclvm_cli.exe run ..\..\samples\kubernetes.k
.\_output\kclvm-windows\bin\kclvm_cli.exe run ..\..\samples\math.k

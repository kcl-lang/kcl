# Copyright The KCL Authors. All rights reserved.

Set-Location $PSScriptRoot
# 1. Install kcl.dll
Set-Location "..\"
cargo build --release
Set-Location $PSScriptRoot

New-Item -ErrorAction Ignore -Path ".\_output" -ItemType "directory"
New-Item -ErrorAction Ignore -Path ".\_output\kcl-core" -ItemType "directory"
New-Item -ErrorAction Ignore -Path ".\_output\kcl-core" -ItemType "directory"

Copy-Item -Path "..\..\target\release\kcl.dll" -Destination ".\_output\kcl-core\kcl.dll" -Force
Copy-Item -Path "..\..\target\release\kcl.dll.lib" -Destination ".\_output\kcl-core\kcl.lib" -Force
Copy-Item -Path "..\..\target\release\kcl.dll.lib" -Destination "..\..\target\release\kcl.lib" -Force

Set-Location $PSScriptRoot
# 2. Install kcl language server
Set-Location "..\"
cargo build --release --manifest-path crates/tools/src/LSP/Cargo.toml
Set-Location $PSScriptRoot
Copy-Item -Path "..\..\target\release\kcl-language-server.exe" -Destination ".\_output\kcl-core\"

Set-Location $PSScriptRoot
# 3. Install libkcl CLI
Set-Location "..\"
cargo build --release --manifest-path crates/cli/Cargo.toml
Set-Location $PSScriptRoot
Copy-Item -Path "..\..\target\release\libkcl.exe" -Destination ".\_output\kcl-core\" -Force

Set-Location $PSScriptRoot

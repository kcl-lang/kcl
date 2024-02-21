Set-Location $PSScriptRoot
. '.\scripts\build-windows\build.ps1'
$bin_path = Join-Path $PSScriptRoot 'scripts\build-windows\_output\kclvm-windows\bin'
$env:Path += ";$bin_path"
# rust unit test
Set-Location .\kclvm
cargo test --workspace -r -- --nocapture
Set-Location $PSScriptRoot
# rust runtime test
Set-Location .\kclvm\tests\test_units
python3 -m pytest -vv
Set-Location $PSScriptRoot
# konfig test
Invoke-Expression -Command '.\test\integration\test_konfig.bat'

# Builds Painty and creates the installer.
Set-Location -Path $PSScriptRoot
$Host.UI.RawUI.WindowTitle = "Painty Build"

cargo build --release
& "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe" "painty.iss"
# Runs Painty from the project folder and sets the terminal tab title.
Set-Location -Path $PSScriptRoot
$Host.UI.RawUI.WindowTitle = "Painty"
cargo run
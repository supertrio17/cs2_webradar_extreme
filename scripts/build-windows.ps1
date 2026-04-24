$ErrorActionPreference = "Stop"

Push-Location "$PSScriptRoot/.."

cargo build -p cs2_webradar_extreme --release --target x86_64-pc-windows-msvc
npm --prefix ./ui ci
npm --prefix ./ui run build

Write-Host "Build complete: target/x86_64-pc-windows-msvc/release/cs2_webradar_extreme.exe"

Pop-Location

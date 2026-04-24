param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Release",
    [ValidateSet("x64")]
    [string]$Platform = "x64",
    [switch]$Clean,
    [switch]$Rebuild
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$targetTriple = "x86_64-pc-windows-msvc"
$profile = if ($Configuration -eq "Release") { "release" } else { "debug" }
$exePath = Join-Path $repoRoot "target/$targetTriple/$profile/cs2_webradar_extreme.exe"
$frontendDist = Join-Path $repoRoot "ui/dist"

Push-Location $repoRoot

try {
    if ($Rebuild) {
        $Clean = $true
    }

    if ($Clean) {
        cargo clean -p cs2_webradar_extreme --target $targetTriple
        if (Test-Path $frontendDist) {
            Remove-Item $frontendDist -Recurse -Force
        }

        if (-not $Rebuild) {
            Write-Host "Clean complete"
            return
        }
    }

    npm --prefix ./ui ci
    npm --prefix ./ui run build

    $cargoArgs = @("build", "-p", "cs2_webradar_extreme", "--target", $targetTriple)
    if ($Configuration -eq "Release") {
        $cargoArgs += "--release"
    }

    cargo @cargoArgs

    Write-Host "Build complete: $exePath"
    Write-Host "Frontend dist embedded from: $frontendDist"
}
finally {
    Pop-Location
}

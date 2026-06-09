$ErrorActionPreference = "Stop"

$InstallDir = "$env:LOCALAPPDATA\dotmage"
$Binary = "$InstallDir\dmage.exe"
$Url = "https://github.com/dotMage/dotmage/releases/latest/download/dmage-windows-x86_64.exe"

Write-Host ""
Write-Host "  .  dotMage installer  ." -ForegroundColor Cyan
Write-Host ""

# Create install directory
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
}

# Download
Write-Host "  Downloading dmage..."
Invoke-WebRequest -Uri $Url -OutFile $Binary -UseBasicParsing

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    Write-Host "  Added to PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "  [ok] dmage installed to $Binary" -ForegroundColor Green
Write-Host ""
Write-Host "  Restart your terminal, then run:"
Write-Host "    dmage auth --server https://your-server:9470"
Write-Host ""

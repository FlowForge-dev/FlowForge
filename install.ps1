$ErrorActionPreference = "Stop"

$Repo = "FlowForge-dev/FlowForge"
$BinDir = if ($env:FLOWFORGE_BIN_DIR) { $env:FLOWFORGE_BIN_DIR } else { "$HOME\.flowforge\bin" }
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("flowforge-" + [System.Guid]::NewGuid())
$Target = "x86_64-pc-windows-msvc"
$Url = "https://github.com/$Repo/releases/latest/download/flowforge-$Target.zip"

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

Write-Host "Downloading FlowForge for Windows..."
Invoke-WebRequest -Uri $Url -OutFile (Join-Path $TmpDir "flowforge.zip")
Expand-Archive -Path (Join-Path $TmpDir "flowforge.zip") -DestinationPath $TmpDir -Force
Copy-Item (Join-Path $TmpDir "forge.exe") (Join-Path $BinDir "forge.exe") -Force
& (Join-Path $BinDir "forge.exe") init

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    Write-Host "Added $BinDir to your user PATH. Open a new terminal."
}

Write-Host "Installed forge to $BinDir\forge.exe"
Write-Host "Run: forge provider list"

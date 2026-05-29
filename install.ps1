$ErrorActionPreference = "Stop"

$Repo = "FlowForge-dev/FlowForge"
$CargoBin = Join-Path $HOME ".cargo\bin"
$BinDir = if ($env:FLOWFORGE_BIN_DIR) { $env:FLOWFORGE_BIN_DIR } else { $CargoBin }
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("flowforge-" + [System.Guid]::NewGuid())
$Target = "x86_64-pc-windows-msvc"
$Url = "https://github.com/$Repo/releases/latest/download/flowforge-$Target.zip"

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

function Add-ToUserPath($PathToAdd) {
    $UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$PathToAdd*") {
        [Environment]::SetEnvironmentVariable("Path", "$UserPath;$PathToAdd", "User")
        $env:Path = "$env:Path;$PathToAdd"
        Write-Host "Added $PathToAdd to your user PATH."
    }
}

$InstalledForge = Join-Path $BinDir "forge.exe"

try {
    Write-Host "Downloading FlowForge for Windows..."
    Invoke-WebRequest -Uri $Url -OutFile (Join-Path $TmpDir "flowforge.zip")
    Expand-Archive -Path (Join-Path $TmpDir "flowforge.zip") -DestinationPath $TmpDir -Force
    Copy-Item (Join-Path $TmpDir "forge.exe") $InstalledForge -Force
    Add-ToUserPath $BinDir
}
catch {
    Write-Host "No release binary found yet. Building from source instead..."
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "Rust is required for source install: https://rustup.rs/"
        throw
    }
    cargo install --git "https://github.com/$Repo" --force
    $InstalledForge = Join-Path $CargoBin "forge.exe"
    Add-ToUserPath $CargoBin
}

& $InstalledForge provider list

Write-Host ""
Write-Host "Installed forge."
Write-Host "Starter files were created automatically in ~/.forgeflow."
Write-Host "Next: run forge config open or forge provider configure openrouter --api-key `"sk-or-...`" --model `"openrouter/free`""

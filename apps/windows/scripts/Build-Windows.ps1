<#
.SYNOPSIS
  Dựng ứng dụng Windows (LichViet.exe), gói .zip và (nếu có WiX 3.14) bộ cài .msi.

.DESCRIPTION
  Chạy được ngay từ bất kỳ đâu trong kho mã:
    powershell -ExecutionPolicy Bypass -File apps\windows\scripts\Build-Windows.ps1

  Cần: Rust (rustup, toolchain MSVC) và Visual Studio Build Tools 2022 (C++). Script tự kiểm tra.
  Bộ cài MSI cần thêm WiX Toolset 3.14 (winget install WiXToolset.WiXToolset) và cargo-wix 0.3.9
  (script tự cài cargo-wix nếu thiếu). Không có WiX thì chỉ tạo .zip.

  Kết quả: apps\windows\dist\LichViet-<phiên bản>-x64.zip (và .msi).
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$App = Split-Path -Parent $PSScriptRoot           # apps\windows
$Dist = Join-Path $App 'dist'

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host 'Chưa có Rust. Cài tại https://rustup.rs (chọn toolchain mặc định x86_64-pc-windows-msvc), rồi mở cửa sổ PowerShell mới.' -ForegroundColor Red
    exit 1
}

Push-Location $App
try {
    # Số phiên bản: tệp VERSION ở gốc kho (build.rs dừng dựng nếu Cargo.toml khác).
    $version = (Get-Content (Join-Path $App "..\..\VERSION") -Raw).Trim()
    Write-Host "== Dựng LichViet $version (release)"
    cargo build --release --locked
    if ($LASTEXITCODE -ne 0) {
        # Lần đầu chưa có Cargo.lock cho crate này: dựng không --locked.
        cargo build --release
        if ($LASTEXITCODE -ne 0) { throw 'cargo build thất bại' }
    }
    $exe = Join-Path $App 'target\release\LichViet.exe'
    if (-not (Test-Path $exe)) { throw "Không thấy $exe" }

    New-Item -ItemType Directory -Force -Path $Dist | Out-Null
    $stage = Join-Path $Dist "LichViet-$version-x64"
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
    New-Item -ItemType Directory -Path $stage | Out-Null
    Copy-Item $exe $stage
    Copy-Item (Join-Path $App 'ui\fonts\OFL.txt') $stage
    $zip = "$stage.zip"
    if (Test-Path $zip) { Remove-Item -Force $zip }
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
    Write-Host "Đã tạo: $zip" -ForegroundColor Green

    $candle = Get-Command candle.exe -ErrorAction SilentlyContinue
    if (-not $candle -and $env:WIX) { $candle = Get-Item (Join-Path $env:WIX 'bin\candle.exe') -ErrorAction SilentlyContinue }
    if ($candle) {
        if (-not (Get-Command cargo-wix -ErrorAction SilentlyContinue)) {
            cargo install cargo-wix --version 0.3.9 --locked
        }
        cargo wix --no-build --nocapture --output (Join-Path $Dist "LichViet-$version-x64.msi")
        if ($LASTEXITCODE -ne 0) { throw 'cargo wix thất bại' }
        Write-Host "Đã tạo: $(Join-Path $Dist "LichViet-$version-x64.msi")" -ForegroundColor Green
    } else {
        Write-Host 'Không thấy WiX Toolset 3.14 nên bỏ qua bộ cài MSI (chỉ có .zip). Cài: winget install WiXToolset.WiXToolset' -ForegroundColor Yellow
    }
}
finally { Pop-Location }

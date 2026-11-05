<#
.SYNOPSIS
  Tạo khóa tải lên (upload key) cho Google Play và in sẵn các giá trị để dán vào GitHub Secrets.

.DESCRIPTION
  Chạy được ngay, không cần sửa gì:
    powershell -ExecutionPolicy Bypass -File .\New-AndroidUploadKeystore.ps1

  - Tệp khóa: %USERPROFILE%\ClevixKeys\lichviet-upload.jks  (NGOÀI thư mục mã nguồn, không bao giờ commit)
  - Bí danh khóa (alias): lichviet-upload
  - Mật khẩu: hỏi bằng Read-Host -AsSecureString (không hiện trên màn hình, không ghi vào lịch sử lệnh)
  - Tạo thêm %USERPROFILE%\ClevixKeys\lichviet-upload.base64.txt  (nội dung cho secret LVHV_ANDROID_KEYSTORE_BASE64)
  - Không ghi đè tệp khóa đã có (nếu có, script dừng lại).

  Cần keytool: có sẵn trong JDK 17+ hoặc trong Android Studio (thư mục jbr\bin). Script tự tìm.

  Sau khi chạy, đặt 4 secret trong GitHub (Settings > Secrets and variables > Actions):
    LVHV_ANDROID_KEYSTORE_BASE64   = nội dung tệp .base64.txt
    LVHV_ANDROID_KEYSTORE_PASSWORD = mật khẩu kho khóa
    LVHV_ANDROID_KEY_ALIAS         = lichviet-upload
    LVHV_ANDROID_KEY_PASSWORD      = mật khẩu khóa (script dùng cùng mật khẩu kho khóa)

  SAO LƯU tệp .jks và mật khẩu ở nơi an toàn (ví dụ USB mã hóa + trình quản lý mật khẩu).
  Mất khóa tải lên thì phải xin Google đặt lại (Play App Signing), mất vài ngày.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Alias = 'lichviet-upload'
$Dir = Join-Path $env:USERPROFILE 'ClevixKeys'
$Jks = Join-Path $Dir 'lichviet-upload.jks'
$B64 = Join-Path $Dir 'lichviet-upload.base64.txt'
$DName = 'CN=Clevix Lab, O=Clevix LLC, L=Ha Noi, C=VN'

function Find-Keytool {
    $cmd = Get-Command keytool -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $candidates = @()
    if ($env:JAVA_HOME) { $candidates += (Join-Path $env:JAVA_HOME 'bin\keytool.exe') }
    $candidates += @(
        (Join-Path $env:ProgramFiles 'Android\Android Studio\jbr\bin\keytool.exe'),
        (Join-Path $env:ProgramFiles 'Android\Android Studio\jre\bin\keytool.exe'),
        (Join-Path $env:LOCALAPPDATA 'Programs\Android Studio\jbr\bin\keytool.exe')
    )
    foreach ($base in @((Join-Path $env:ProgramFiles 'Java'), (Join-Path $env:ProgramFiles 'Eclipse Adoptium'), (Join-Path $env:ProgramFiles 'Microsoft'))) {
        if (Test-Path $base) {
            Get-ChildItem -Path $base -Directory -ErrorAction SilentlyContinue | ForEach-Object {
                $candidates += (Join-Path $_.FullName 'bin\keytool.exe')
            }
        }
    }
    foreach ($c in $candidates) { if (Test-Path $c) { return $c } }
    return $null
}

function ConvertTo-Plain([Security.SecureString]$s) {
    $b = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($s)
    try { return [Runtime.InteropServices.Marshal]::PtrToStringBSTR($b) }
    finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($b) }
}

$keytool = Find-Keytool
if (-not $keytool) {
    Write-Host 'Không tìm thấy keytool. Cài JDK 17 (ví dụ: winget install Microsoft.OpenJDK.17) hoặc Android Studio, rồi chạy lại.' -ForegroundColor Red
    exit 1
}
Write-Host "Dùng keytool: $keytool"

if (Test-Path $Jks) {
    Write-Host "Đã có tệp khóa: $Jks" -ForegroundColor Yellow
    Write-Host 'Script không ghi đè khóa cũ. Nếu thật sự muốn tạo lại, tự đổi tên hoặc chuyển tệp cũ đi chỗ khác trước.'
    exit 1
}
New-Item -ItemType Directory -Force -Path $Dir | Out-Null

$p1 = Read-Host -AsSecureString 'Đặt mật khẩu cho kho khóa (ít nhất 12 ký tự)'
$p2 = Read-Host -AsSecureString 'Nhập lại mật khẩu'
$pw = ConvertTo-Plain $p1
$pw2 = ConvertTo-Plain $p2
try {
    if ($pw -ne $pw2) { Write-Host 'Hai lần nhập không khớp. Chạy lại script.' -ForegroundColor Red; exit 1 }
    if ($pw.Length -lt 12) { Write-Host 'Mật khẩu quá ngắn (cần ít nhất 12 ký tự).' -ForegroundColor Red; exit 1 }

    # Truyền mật khẩu qua biến môi trường tạm (:env) thay vì dòng lệnh, để không lộ trong danh sách tiến trình.
    $env:LVHV_KS_PW = $pw
    & $keytool -genkeypair -v `
        -keystore $Jks -storetype PKCS12 `
        -alias $Alias -keyalg RSA -keysize 4096 -validity 10000 `
        -dname $DName `
        -storepass:env LVHV_KS_PW -keypass:env LVHV_KS_PW
    if ($LASTEXITCODE -ne 0) { throw "keytool lỗi (mã $LASTEXITCODE)" }
}
finally {
    Remove-Item Env:\LVHV_KS_PW -ErrorAction SilentlyContinue
    $pw = $null; $pw2 = $null
}

$bytes = [IO.File]::ReadAllBytes($Jks)
[IO.File]::WriteAllText($B64, [Convert]::ToBase64String($bytes))

Write-Host ''
Write-Host 'Đã tạo xong.' -ForegroundColor Green
Write-Host "  Tệp khóa : $Jks"
Write-Host "  Base64   : $B64"
Write-Host ''
Write-Host 'Đặt các GitHub secret sau (Settings > Secrets and variables > Actions > New repository secret):'
Write-Host '  LVHV_ANDROID_KEYSTORE_BASE64   = toàn bộ nội dung tệp Base64 ở trên (đã chép vào clipboard)'
Write-Host '  LVHV_ANDROID_KEYSTORE_PASSWORD = mật khẩu vừa đặt'
Write-Host "  LVHV_ANDROID_KEY_ALIAS         = $Alias"
Write-Host '  LVHV_ANDROID_KEY_PASSWORD      = mật khẩu vừa đặt (cùng mật khẩu)'
try {
    Get-Content -Raw $B64 | Set-Clipboard
    Write-Host 'Nội dung Base64 đã được chép vào clipboard.'
} catch {
    Write-Host 'Không chép được vào clipboard; mở tệp Base64 bằng Notepad để chép.'
}
Write-Host ''
Write-Host 'Dấu vân tay chứng chỉ (dùng khi khai báo với Google Play):'
$env:LVHV_KS_PW2 = ConvertTo-Plain $p1
try {
    $fp = & $keytool -list -v -keystore $Jks -alias $Alias -storepass:env LVHV_KS_PW2 2>$null | Where-Object { $_ -match 'SHA256:' }
    $fp | ForEach-Object { Write-Host $_.Trim() }
}
finally { Remove-Item Env:\LVHV_KS_PW2 -ErrorAction SilentlyContinue }
Write-Host ''
Write-Host 'NHỚ SAO LƯU tệp .jks và mật khẩu. Tệp .base64.txt có thể xóa sau khi đã đặt secret.' -ForegroundColor Yellow

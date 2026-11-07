<#
.SYNOPSIS
  Tạo khóa ký bản phát hành Android của Lịch và Văn hóa Việt và đặt 4 secret LVHV_ANDROID_* cho CI.

.DESCRIPTION
  Chạy được ngay, không cần sửa gì. Mở PowerShell rồi chạy (đứng ở thư mục nào cũng được):

    powershell -ExecutionPolicy Bypass -File .\scripts\tao-khoa-ky-android.ps1

  Script làm lần lượt:
    1. Kiểm tra có keytool (đi kèm JDK) và GitHub CLI (gh), đã đăng nhập gh, có quyền quản trị kho
       ClevixLab/lich-van-hoa-viet. Thiếu gì thì in đúng lệnh cài rồi dừng.
    2. Tạo kho khóa %USERPROFILE%\ClevixLab\khoa-ky-android\lichviet-release.jks (PKCS12, RSA 4096,
       hạn 10 000 ngày, bí danh "lichviet"). Mật khẩu hỏi bằng Read-Host -AsSecureString (không hiện trên
       màn hình, không vào lịch sử lệnh). KHÔNG BAO GIỜ ghi đè tệp khóa đã có: nếu đã có, script dùng lại khóa
       đó (hỏi mật khẩu để kiểm tra) thay vì tạo khóa mới.
    3. In dấu vân tay SHA-256 của chứng chỉ, ghi thông tin khóa (không có mật khẩu) vào thong-tin-khoa.txt
       cùng thư mục.
    4. Đặt 4 secret của kho bằng `gh secret set` (giá trị đưa qua đầu vào chuẩn, không nằm trên dòng lệnh):
         LVHV_ANDROID_KEYSTORE_BASE64   = nội dung tệp .jks mã hóa Base64
         LVHV_ANDROID_KEYSTORE_PASSWORD = mật khẩu kho khóa
         LVHV_ANDROID_KEY_ALIAS         = lichviet
         LVHV_ANDROID_KEY_PASSWORD      = mật khẩu khóa (với PKCS12 bằng mật khẩu kho khóa)
       Đây đúng là tên mà .github/workflows/android.yml đọc.
    5. Nhắc SAO LƯU tệp khóa và mật khẩu.

  Mọi thao tác ghi tệp chỉ nằm trong %USERPROFILE%\ClevixLab\khoa-ky-android\. Script không ghi gì vào thư mục
  mã nguồn và không đụng tới tệp nào khác.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch { }

$Repo = 'ClevixLab/lich-van-hoa-viet'
$Dir = Join-Path $env:USERPROFILE 'ClevixLab\khoa-ky-android'
$Jks = Join-Path $Dir 'lichviet-release.jks'
$Info = Join-Path $Dir 'thong-tin-khoa.txt'
$Alias = 'lichviet'
$DName = 'CN=Clevix Lab, OU=Lich va Van hoa Viet, O=Clevix Lab, C=VN'
$SecretNames = @('LVHV_ANDROID_KEYSTORE_BASE64', 'LVHV_ANDROID_KEYSTORE_PASSWORD', 'LVHV_ANDROID_KEY_ALIAS', 'LVHV_ANDROID_KEY_PASSWORD')
$MinLen = 12

function Wait-Close {
    Write-Host ''
    try { [void](Read-Host 'Nhấn Enter để đóng') } catch { }
}

function Stop-Script([string]$Message) {
    Write-Host ''
    Write-Host $Message -ForegroundColor Red
    Wait-Close
    exit 1
}

function Find-Keytool {
    $cmd = Get-Command keytool -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    $candidates = @()
    if ($env:JAVA_HOME) { $candidates += (Join-Path $env:JAVA_HOME 'bin\keytool.exe') }
    $candidates += @(
        (Join-Path $env:ProgramFiles 'Android\Android Studio\jbr\bin\keytool.exe'),
        (Join-Path $env:LOCALAPPDATA 'Programs\Android Studio\jbr\bin\keytool.exe')
    )
    foreach ($base in @((Join-Path $env:ProgramFiles 'Eclipse Adoptium'), (Join-Path $env:ProgramFiles 'Java'), (Join-Path $env:ProgramFiles 'Microsoft'))) {
        if (Test-Path -LiteralPath $base) {
            Get-ChildItem -LiteralPath $base -Directory -ErrorAction SilentlyContinue | ForEach-Object {
                $candidates += (Join-Path $_.FullName 'bin\keytool.exe')
            }
        }
    }
    foreach ($c in $candidates) { if (Test-Path -LiteralPath $c) { return $c } }
    return $null
}

function Find-Gh {
    $cmd = Get-Command gh -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    foreach ($c in @((Join-Path $env:ProgramFiles 'GitHub CLI\gh.exe'), (Join-Path $env:LOCALAPPDATA 'Programs\GitHub CLI\gh.exe'))) {
        if (Test-Path -LiteralPath $c) { return $c }
    }
    return $null
}

function ConvertTo-Plain([Security.SecureString]$s) {
    $b = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($s)
    try { return [Runtime.InteropServices.Marshal]::PtrToStringBSTR($b) }
    finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($b) }
}

# Chạy một chương trình ngoài, gom cả stdout và stderr thành chuỗi. Windows PowerShell 5.1 coi mỗi dòng stderr của
# chương trình ngoài là lỗi (keytool và gh in thông báo thường ra stderr), nên trong hàm này không dừng vì stderr; chỉ
# xét mã thoát.
function Invoke-Native([string]$Exe, [string[]]$ArgList) {
    $ErrorActionPreference = 'Continue'
    $out = & $Exe @ArgList 2>&1
    $code = $LASTEXITCODE
    return @{ Code = $code; Out = @($out | ForEach-Object { "$_" }) }
}

# Dòng báo lỗi cuối của chương trình ngoài (bỏ dòng "Picked up JAVA_TOOL_OPTIONS" của Java).
function Get-LastLine([string[]]$Lines) {
    $l = @($Lines | Where-Object { $_ -and $_ -notmatch '^Picked up ' })
    if ($l.Count -eq 0) { return '(không có thông báo)' }
    return $l[-1].Trim()
}

# Chạy keytool với mật khẩu qua biến môi trường tạm (-storepass:env), không qua dòng lệnh.
function Invoke-Keytool([string]$Password, [string[]]$KeytoolArgs) {
    $env:LVHV_KS_PW = $Password
    try { return (Invoke-Native $script:Keytool $KeytoolArgs) }
    finally { Remove-Item Env:\LVHV_KS_PW -ErrorAction SilentlyContinue }
}

# Đặt một secret: giá trị ghi đúng từng byte (UTF-8, không xuống dòng) vào đầu vào chuẩn của gh.
function Set-RepoSecret([string]$Name, [string]$Value) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $script:Gh
    $psi.Arguments = "secret set $Name --repo $Repo"
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $p = [System.Diagnostics.Process]::Start($psi)
    $bytes = (New-Object System.Text.UTF8Encoding($false)).GetBytes($Value)
    $p.StandardInput.BaseStream.Write($bytes, 0, $bytes.Length)
    $p.StandardInput.BaseStream.Flush()
    $p.StandardInput.Close()
    $errTask = $p.StandardError.ReadToEndAsync()
    [void]$p.StandardOutput.ReadToEnd()
    $p.WaitForExit()
    $err = $errTask.Result
    if ($p.ExitCode -ne 0) { Stop-Script "Không đặt được secret $Name (gh báo: $($err.Trim()))." }
    Write-Host "  Đã đặt $Name" -ForegroundColor Green
}

Write-Host '=== Tạo khóa ký Android — Lịch và Văn hóa Việt ===' -ForegroundColor Cyan
Write-Host "Thư mục khóa: $Dir"
Write-Host ''

# ---------- 1. Công cụ cần có ----------
$Keytool = Find-Keytool
if (-not $Keytool) {
    Stop-Script (@(
        'Không tìm thấy keytool (đi kèm bộ JDK).',
        'Cài JDK 17 bằng lệnh sau trong PowerShell (Windows 10/11 có sẵn winget):',
        '',
        '    winget install --id EclipseAdoptium.Temurin.17.JDK -e',
        '',
        'Cài xong, ĐÓNG cửa sổ PowerShell này, mở cửa sổ mới rồi chạy lại script.'
    ) -join "`n")
}
Write-Host "keytool : $Keytool"

$Gh = Find-Gh
if (-not $Gh) {
    Stop-Script (@(
        'Không tìm thấy GitHub CLI (gh).',
        'Cài bằng lệnh:',
        '',
        '    winget install --id GitHub.cli -e',
        '',
        'Cài xong, ĐÓNG cửa sổ PowerShell này, mở cửa sổ mới, đăng nhập một lần:',
        '',
        '    gh auth login',
        '',
        '(chọn GitHub.com → HTTPS → Login with a web browser), rồi chạy lại script.'
    ) -join "`n")
}
Write-Host "gh      : $Gh"

$r = Invoke-Native $Gh @('auth', 'status', '--hostname', 'github.com')
if ($r.Code -ne 0) {
    Stop-Script (@(
        'GitHub CLI chưa đăng nhập. Chạy lệnh sau, chọn GitHub.com → HTTPS → Login with a web browser:',
        '',
        '    gh auth login',
        '',
        'rồi chạy lại script.'
    ) -join "`n")
}
$r = Invoke-Native $Gh @('api', "repos/$Repo", '--jq', '.permissions.admin')
if ($r.Code -ne 0) {
    Stop-Script "Tài khoản GitHub đang đăng nhập không mở được kho $Repo ($(($r.Out | Select-Object -First 1))). Đăng nhập đúng tài khoản chủ dự án: gh auth login"
}
if ((($r.Out -join '').Trim()) -ne 'true') {
    Stop-Script "Tài khoản GitHub đang đăng nhập không có quyền quản trị (admin) kho $Repo — cần quyền này để đặt secret. Đăng nhập đúng tài khoản chủ dự án: gh auth login"
}
Write-Host "Kho     : $Repo (có quyền quản trị)"

$old = Join-Path $env:USERPROFILE 'ClevixKeys\lichviet-upload.jks'
if (Test-Path -LiteralPath $old) {
    Write-Host ''
    Write-Host "Lưu ý: máy này có khóa cũ $old (script cũ apps\windows\scripts\New-AndroidUploadKeystore.ps1)." -ForegroundColor Yellow
    Write-Host 'Script này KHÔNG đọc, không sửa, không xóa khóa đó; bản phát hành Android sẽ ký bằng khóa trong thư mục mới ở trên.' -ForegroundColor Yellow
}

# ---------- 2. Kho khóa (không bao giờ ghi đè) ----------
$created = $false
$pw = $null
try {
    if (Test-Path -LiteralPath $Jks) {
        Write-Host ''
        Write-Host "Đã có tệp khóa: $Jks" -ForegroundColor Yellow
        Write-Host 'Script KHÔNG tạo khóa mới và không ghi đè. Sẽ dùng lại khóa này để đặt secret.'
        $pw = ConvertTo-Plain (Read-Host -AsSecureString 'Mật khẩu của kho khóa đã có')
        $r = Invoke-Keytool $pw @('-list', '-keystore', $Jks, '-storetype', 'PKCS12', '-alias', $Alias, '-storepass:env', 'LVHV_KS_PW')
        if ($r.Code -ne 0) { Stop-Script "Mật khẩu sai hoặc kho khóa không có bí danh '$Alias' (keytool: $(Get-LastLine $r.Out)). Chạy lại script và nhập đúng mật khẩu đã đặt khi tạo khóa." }
    }
    else {
        Write-Host ''
        Write-Host "Đặt mật khẩu cho khóa ký (ít nhất $MinLen ký tự, chỉ chữ KHÔNG DẤU, số và ký hiệu bàn phím)."
        Write-Host 'Mật khẩu này dùng cho cả kho khóa và khóa (định dạng PKCS12 chỉ có một mật khẩu).'
        $p1 = ConvertTo-Plain (Read-Host -AsSecureString 'Mật khẩu')
        $p2 = ConvertTo-Plain (Read-Host -AsSecureString 'Nhập lại mật khẩu')
        if ($p1 -cne $p2) { $p1 = $null; $p2 = $null; Stop-Script 'Hai lần nhập không khớp. Chạy lại script.' }
        $p2 = $null
        if ($p1.Length -lt $MinLen) { $p1 = $null; Stop-Script "Mật khẩu quá ngắn (cần ít nhất $MinLen ký tự). Chạy lại script." }
        if ($p1 -notmatch '^[\x21-\x7E]+$') { $p1 = $null; Stop-Script 'Mật khẩu chỉ được có chữ không dấu, số và ký hiệu bàn phím (không dấu cách, không chữ có dấu). Chạy lại script.' }
        $pw = $p1; $p1 = $null

        New-Item -ItemType Directory -Force -Path $Dir | Out-Null
        $r = Invoke-Keytool $pw @(
            '-genkeypair', '-keystore', $Jks, '-storetype', 'PKCS12',
            '-alias', $Alias, '-keyalg', 'RSA', '-keysize', '4096', '-validity', '10000',
            '-dname', $DName, '-storepass:env', 'LVHV_KS_PW', '-keypass:env', 'LVHV_KS_PW'
        )
        if ($r.Code -ne 0 -or -not (Test-Path -LiteralPath $Jks)) { Stop-Script "keytool không tạo được khóa: $(Get-LastLine $r.Out)" }
        $created = $true
        Write-Host "Đã tạo khóa: $Jks" -ForegroundColor Green
    }

    # ---------- 3. Dấu vân tay ----------
    $r = Invoke-Keytool $pw @('-list', '-v', '-keystore', $Jks, '-storetype', 'PKCS12', '-alias', $Alias, '-storepass:env', 'LVHV_KS_PW')
    $fpLine = $r.Out | Where-Object { $_ -match '^\s*SHA256:\s*' } | Select-Object -First 1
    if (-not $fpLine) { Stop-Script 'Không đọc được dấu vân tay SHA-256 của chứng chỉ.' }
    $fp = ($fpLine -replace '^\s*SHA256:\s*', '').Trim()
    $validLine = ($r.Out | Where-Object { $_ -match 'Valid from|Có hiệu lực' } | Select-Object -First 1)
    Write-Host ''
    Write-Host 'Dấu vân tay chứng chỉ (SHA-256):' -ForegroundColor Cyan
    Write-Host "  $fp"
    Write-Host '(CI in cùng dấu vân tay này, viết thường không dấu hai chấm, ở tóm tắt run "android" và ghi chú bản phát hành.)'

    if ($created -or -not (Test-Path -LiteralPath $Info)) {
        $lines = @(
            'Khóa ký bản phát hành Android — Lịch và Văn hóa Việt (vn.clevix.lichviet)',
            "Tệp khóa      : $Jks",
            'Định dạng     : PKCS12, RSA 4096, hạn 10 000 ngày',
            "Bí danh       : $Alias",
            "Chủ thể       : $DName",
            "SHA-256       : $fp",
            "Hiệu lực      : $(if ($validLine) { $validLine.Trim() } else { '(xem keytool -list -v)' })",
            "Tạo/ghi lúc   : $((Get-Date).ToString('yyyy-MM-dd HH:mm'))",
            "Secret GitHub : $($SecretNames -join ', ') trên $Repo",
            'Mật khẩu      : KHÔNG ghi ở đây — giữ trong trình quản lý mật khẩu.',
            '',
            'MẤT tệp khóa hoặc mật khẩu = KHÔNG BAO GIỜ phát hành được bản cập nhật cho người đã cài ứng dụng từ tệp APK.'
        )
        [IO.File]::WriteAllLines($Info, $lines, (New-Object System.Text.UTF8Encoding($true)))
        Write-Host "Đã ghi thông tin khóa (không có mật khẩu): $Info"
    }

    # ---------- 4. Secret GitHub ----------
    $r = Invoke-Native $Gh @('secret', 'list', '--repo', $Repo)
    if ($r.Code -ne 0) { Stop-Script "Không đọc được danh sách secret của $Repo ($(($r.Out | Select-Object -First 1)))." }
    $existing = @($r.Out | ForEach-Object { ("$_".Trim() -split '\s+')[0] } | Where-Object { $SecretNames -contains $_ })
    Write-Host ''
    if ($existing.Count -gt 0) {
        Write-Host "Kho đã có secret: $($existing -join ', ')." -ForegroundColor Yellow
        Write-Host 'Đặt lại sẽ THAY các giá trị đó bằng khóa ở trên (giá trị cũ không xem lại được).' -ForegroundColor Yellow
        $ans = Read-Host 'Gõ CO (chữ in hoa) để thay, phím khác để dừng'
        if ($ans -cne 'CO') { Stop-Script 'Đã dừng, không đổi secret nào. Tệp khóa vẫn còn nguyên trong thư mục khóa.' }
    }
    Write-Host "Đặt 4 secret cho $Repo ..."
    $b64 = [Convert]::ToBase64String([IO.File]::ReadAllBytes($Jks))
    Set-RepoSecret 'LVHV_ANDROID_KEYSTORE_BASE64' $b64
    Set-RepoSecret 'LVHV_ANDROID_KEYSTORE_PASSWORD' $pw
    Set-RepoSecret 'LVHV_ANDROID_KEY_ALIAS' $Alias
    Set-RepoSecret 'LVHV_ANDROID_KEY_PASSWORD' $pw
    $b64 = $null
}
finally {
    $pw = $null
    Remove-Item Env:\LVHV_KS_PW -ErrorAction SilentlyContinue
}

# ---------- 5. Nhắc sao lưu ----------
Write-Host ''
Write-Host '======================================================================' -ForegroundColor Red
Write-Host ' QUAN TRỌNG — SAO LƯU NGAY KHÓA KÝ VÀ MẬT KHẨU' -ForegroundColor Red
Write-Host '======================================================================' -ForegroundColor Red
Write-Host " Tệp khóa: $Jks" -ForegroundColor Yellow
Write-Host ' Mọi bản cập nhật Android phải ký bằng ĐÚNG khóa này. Nếu mất tệp khóa' -ForegroundColor Yellow
Write-Host ' HOẶC quên mật khẩu thì KHÔNG BAO GIỜ cập nhật được ứng dụng cho người đã' -ForegroundColor Yellow
Write-Host ' cài: họ phải gỡ ứng dụng (mất dữ liệu nếu chưa sao lưu) rồi cài lại.' -ForegroundColor Yellow
Write-Host ' Secret trên GitHub không tải về lại được, nên KHÔNG thay được bản sao lưu.' -ForegroundColor Yellow
Write-Host ''
Write-Host ' Làm ngay:' -ForegroundColor Yellow
Write-Host '  1. Chép cả thư mục khóa ra ít nhất 2 nơi tách biệt (ví dụ USB mã hóa' -ForegroundColor Yellow
Write-Host '     cất ở nhà + ổ đĩa mã hóa khác), không để trong thư mục mã nguồn.' -ForegroundColor Yellow
Write-Host '  2. Lưu mật khẩu trong trình quản lý mật khẩu (và một bản giấy cất kỹ).' -ForegroundColor Yellow
Write-Host '  3. Không gửi tệp khóa hay mật khẩu qua email/chat, không commit vào git.' -ForegroundColor Yellow
Write-Host '======================================================================' -ForegroundColor Red
Write-Host ''
Write-Host 'Bước tiếp theo — đăng bản phát hành Android (sau khi PR release-android-0.2.1 đã gộp vào main):' -ForegroundColor Cyan
Write-Host "  gh workflow run android.yml --repo $Repo --ref main -f publish_release=true"
Write-Host '  (hoặc GitHub → Actions → android → Run workflow → nhánh main → đánh dấu publish_release)'
Write-Host '  Job "publish" đăng LichViet-<VERSION>.apk và .aab lên bản phát hành v<VERSION> sau khi mọi kiểm thử đạt.'
Wait-Close

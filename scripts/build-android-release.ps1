#Requires -Version 5.1
<#
.SYNOPSIS
  Build a release (formal) arm64-v8a APK for CodeOrbit Companion.

.DESCRIPTION
  - Frontend production build
  - Rust release (strip/LTO via Cargo.toml [profile.release])
  - Copy .so into jniLibs (avoids Windows symlink requirement)
  - Gradle assembleArm64Release with minify; skip rustBuild*
  - Sign: RELEASE_KEYSTORE if set, else Android debug keystore (sideload OK)

.OUTPUT
  dist-android/codeorbit-client-arm64-release.apk

.ENV
  ANDROID_HOME / ANDROID_SDK_ROOT, JAVA_HOME, NDK_HOME (optional auto-detect)
  RELEASE_KEYSTORE, RELEASE_KEYSTORE_PASSWORD, RELEASE_KEY_ALIAS, RELEASE_KEY_PASSWORD
#>
$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

function Write-Step([string]$msg) { Write-Host "`n==> $msg" -ForegroundColor Cyan }
function Die([string]$msg) { Write-Host "ERROR: $msg" -ForegroundColor Red; exit 1 }

# --- env ---
if (-not $env:ANDROID_HOME) {
  $defaultSdk = Join-Path $env:LOCALAPPDATA "Android\Sdk"
  if (Test-Path $defaultSdk) { $env:ANDROID_HOME = $defaultSdk }
}
if (-not $env:ANDROID_HOME) { Die "ANDROID_HOME not set and default SDK missing" }
$env:ANDROID_SDK_ROOT = $env:ANDROID_HOME

if (-not $env:JAVA_HOME) {
  $jdk = "C:\Program Files\Java\jdk-17"
  if (Test-Path $jdk) { $env:JAVA_HOME = $jdk }
}
if (-not $env:JAVA_HOME) { Die "JAVA_HOME not set" }

if (-not $env:NDK_HOME) {
  $ndkRoot = Join-Path $env:ANDROID_HOME "ndk"
  if (Test-Path $ndkRoot) {
    $latest = Get-ChildItem $ndkRoot -Directory | Sort-Object Name -Descending | Select-Object -First 1
    if ($latest) { $env:NDK_HOME = $latest.FullName }
  }
}
if (-not $env:NDK_HOME) { Die "NDK_HOME not set / no NDK under SDK" }

$env:Path = "$($env:JAVA_HOME)\bin;$($env:ANDROID_HOME)\platform-tools;$($env:ANDROID_HOME)\cmdline-tools\latest\bin;$env:Path"

$Abi = "arm64-v8a"
$RustTarget = "aarch64-linux-android"
$SoName = "libcodeorbit_client_lib.so"
$AndroidDir = Join-Path $Root "src-tauri\gen\android"
$JniDir = Join-Path $AndroidDir "app\src\main\jniLibs\$Abi"
$SoRelease = Join-Path $Root "src-tauri\target\$RustTarget\release\$SoName"
$OutDir = Join-Path $Root "dist-android"
$OutApk = Join-Path $OutDir "codeorbit-client-arm64-release.apk"

if (-not (Test-Path $AndroidDir)) {
  Die "Android project missing: src-tauri/gen/android (run: npx tauri android init)"
}

# --- ensure portrait orientation (gen/android may regenerate) ---
Write-Step "Ensure AndroidManifest portrait orientation"
$ManifestPath = Join-Path $AndroidDir "app\src\main\AndroidManifest.xml"
if (-not (Test-Path $ManifestPath)) {
  Die "AndroidManifest missing: $ManifestPath"
}
$manifestText = Get-Content -Raw -Path $ManifestPath
if ($manifestText -notmatch 'android:screenOrientation\s*=\s*"portrait"') {
  if ($manifestText -match '(<activity\b[^>]*android:name="\.MainActivity"[^>]*)(/?>)') {
    $updated = $manifestText -replace '(<activity\b[^>]*android:name="\.MainActivity"[^>]*)(/?>)', '$1 android:screenOrientation="portrait"$2'
    if ($updated -eq $manifestText) {
      # fallback: inject before closing > of first activity (regex once via MatchEvaluator)
      $rx = [regex]'(?s)(<activity\b[^>]*?)(\s*/?>)'
      $updated = $rx.Replace($manifestText, {
        param($m)
        $m.Groups[1].Value + ' android:screenOrientation="portrait"' + $m.Groups[2].Value
      }, 1)
    }
    if ($updated -eq $manifestText -or $updated -notmatch 'android:screenOrientation\s*=\s*"portrait"') {
      Die "Could not patch MainActivity with android:screenOrientation=portrait"
    }
    [System.IO.File]::WriteAllText($ManifestPath, $updated)
    Write-Host "  patched screenOrientation=portrait"
  } else {
    Die "Could not patch MainActivity with android:screenOrientation=portrait"
  }
} else {
  Write-Host "  portrait already set"
}

# --- local.properties ---
Write-Step "Write local.properties"
$sdkPosix = ($env:ANDROID_HOME -replace '\\', '/')
Set-Content -Path (Join-Path $AndroidDir "local.properties") -Value "sdk.dir=$sdkPosix" -Encoding ascii

# --- frontend ---
Write-Step "Frontend production build"
npm run build
if ($LASTEXITCODE -ne 0) { Die "npm run build failed" }

# --- rust release via tauri (configures NDK); ignore Windows symlink failure ---
Write-Step "Rust release ($RustTarget)"
$prevEap = $ErrorActionPreference
$ErrorActionPreference = "Continue"
npx tauri android build --apk --target aarch64 --ci 2>&1 | Write-Host
$tauriCode = $LASTEXITCODE
$ErrorActionPreference = $prevEap

if ($tauriCode -eq 0 -and (Test-Path $OutApk) -eq $false) {
  # tauri may have written under gen/android outputs already
  Write-Host "tauri android build exit 0"
} elseif ($tauriCode -ne 0) {
  Write-Host "tauri android build incomplete (common on Windows without Developer Mode). Using staged .so + Gradle..." -ForegroundColor Yellow
}

if (-not (Test-Path $SoRelease)) {
  Die "Release .so not found: $SoRelease — NDK/linker/cargo release failed"
}

# --- stage jni lib (copy, not symlink) ---
Write-Step "Stage release .so -> jniLibs/$Abi"
New-Item -ItemType Directory -Force -Path $JniDir | Out-Null
Copy-Item -Force $SoRelease (Join-Path $JniDir $SoName)
$soSize = (Get-Item (Join-Path $JniDir $SoName)).Length
Write-Host ("  {0:N1} MB  {1}" -f ($soSize / 1MB), $SoName)
if ($soSize -gt 40MB) {
  Write-Host "  WARN: .so still large; expected ~5-20MB for stripped release" -ForegroundColor Yellow
}

$strip = Get-ChildItem -Path (Join-Path $env:NDK_HOME "toolchains\llvm\prebuilt") -Recurse -Filter "llvm-strip.exe" -ErrorAction SilentlyContinue |
  Select-Object -First 1
if ($strip) {
  Write-Step "llvm-strip"
  & $strip.FullName (Join-Path $JniDir $SoName)
  $soSize = (Get-Item (Join-Path $JniDir $SoName)).Length
  Write-Host ("  after strip: {0:N1} MB" -f ($soSize / 1MB))
}

# --- signing init script (does not mutate gen sources permanently beyond this run) ---
Write-Step "Prepare release signing (Gradle init)"
$initGradle = Join-Path $env:TEMP "codeorbit-android-release-signing.gradle"
if ($env:RELEASE_KEYSTORE -and (Test-Path $env:RELEASE_KEYSTORE)) {
  if (-not $env:RELEASE_KEYSTORE_PASSWORD -or -not $env:RELEASE_KEY_ALIAS) {
    Die "RELEASE_KEYSTORE set but RELEASE_KEYSTORE_PASSWORD / RELEASE_KEY_ALIAS missing"
  }
  $keyPass = if ($env:RELEASE_KEY_PASSWORD) { $env:RELEASE_KEY_PASSWORD } else { $env:RELEASE_KEYSTORE_PASSWORD }
  $storeEsc = ($env:RELEASE_KEYSTORE -replace '\\', '/')
  $initBody = @"
gradle.projectsLoaded {
  rootProject.subprojects { sub ->
    sub.afterEvaluate {
      if (sub.name != 'app' || !sub.plugins.hasPlugin('com.android.application')) return
      def android = sub.extensions.findByName('android')
      if (android == null) return
      android.signingConfigs {
        releaseCi {
          storeFile file('$storeEsc')
          storePassword '$($env:RELEASE_KEYSTORE_PASSWORD)'
          keyAlias '$($env:RELEASE_KEY_ALIAS)'
          keyPassword '$keyPass'
        }
      }
      android.buildTypes.release.signingConfig = android.signingConfigs.releaseCi
    }
  }
}
"@
  Write-Host "  using RELEASE_KEYSTORE"
} else {
  $initBody = @"
gradle.projectsLoaded {
  rootProject.subprojects { sub ->
    sub.afterEvaluate {
      if (sub.name != 'app' || !sub.plugins.hasPlugin('com.android.application')) return
      def android = sub.extensions.findByName('android')
      if (android == null) return
      // installable sideload release; not Play Store ready
      android.buildTypes.release.signingConfig = android.signingConfigs.debug
    }
  }
}
"@
  Write-Host "  no RELEASE_KEYSTORE — debug-signed release (sideload OK)" -ForegroundColor Yellow
}
Set-Content -Path $initGradle -Value $initBody -Encoding ascii

# --- gradle release ---
Write-Step "Gradle assembleArm64Release"
$gradlew = Join-Path $AndroidDir "gradlew.bat"
Push-Location $AndroidDir
try {
  & $gradlew `
    ":app:assembleArm64Release" `
    "--no-daemon" `
    "-I" $initGradle `
    "-x" "rustBuildArm64Release"
  if ($LASTEXITCODE -ne 0) { Die "Gradle release build failed" }
} finally {
  Pop-Location
  Remove-Item -Force $initGradle -ErrorAction SilentlyContinue
}

# --- collect apk ---
Write-Step "Collect APK"
$apkRoot = Join-Path $AndroidDir "app\build\outputs\apk"
$built = Get-ChildItem -Path $apkRoot -Recurse -Filter "*.apk" -ErrorAction SilentlyContinue |
  Where-Object { $_.Name -match 'release' -and $_.FullName -match 'arm64' } |
  Sort-Object LastWriteTime -Descending |
  Select-Object -First 1
if (-not $built) {
  $built = Get-ChildItem -Path $apkRoot -Recurse -Filter "*release*.apk" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
}
if (-not $built) { Die "No release APK under app/build/outputs/apk" }

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Copy-Item -Force $built.FullName $OutApk
$apkSize = (Get-Item $OutApk).Length
Write-Host ""
Write-Host "OK  $OutApk" -ForegroundColor Green
Write-Host ("    size: {0:N1} MB" -f ($apkSize / 1MB))
Write-Host "    install: adb install -r dist-android/codeorbit-client-arm64-release.apk"
if (-not $env:RELEASE_KEYSTORE) {
  Write-Host "    note: debug-signed. Store upload needs RELEASE_KEYSTORE + passwords." -ForegroundColor Yellow
}
if ($apkSize -gt 40MB) {
  Write-Host "    WARN: APK still large; check that libcodeorbit_client_lib.so is release+stripped" -ForegroundColor Yellow
  exit 2
}

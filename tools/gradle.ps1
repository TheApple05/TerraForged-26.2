[CmdletBinding(PositionalBinding = $false)]
param(
    [string] $JavaHome,
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]] $GradleArgs
)

$ErrorActionPreference = 'Stop'
$workspaceRoot = Split-Path -Parent $PSScriptRoot
$jdkCandidates = @($JavaHome, $env:TF_JAVA_HOME, $env:JAVA_HOME)
$localJavaRoot = Join-Path $env:LOCALAPPDATA 'Programs\Java'
if (Test-Path -LiteralPath $localJavaRoot) {
    $jdkCandidates += Get-ChildItem -LiteralPath $localJavaRoot -Directory |
        Sort-Object Name -Descending | ForEach-Object { $_.FullName }
}

$selectedJdk = $null
foreach ($candidate in $jdkCandidates) {
    if ([string]::IsNullOrWhiteSpace($candidate)) { continue }
    $releaseFile = Join-Path $candidate 'release'
    $javacExe = Join-Path $candidate 'bin\javac.exe'
    if ((Test-Path -LiteralPath $releaseFile) -and (Test-Path -LiteralPath $javacExe)) {
        if (Get-Content -LiteralPath $releaseFile | Select-String '^JAVA_VERSION="25(?:[."]|$)') {
            $selectedJdk = (Resolve-Path -LiteralPath $candidate).Path
            break
        }
    }
}

if (-not $selectedJdk) {
    throw 'A full Java 25 JDK is required. Set TF_JAVA_HOME or pass -JavaHome with its directory.'
}
if (-not $GradleArgs) { $GradleArgs = @('help') }

$previousJavaHome = $env:JAVA_HOME
$previousPath = $env:PATH
$gradleExitCode = 1
Push-Location -LiteralPath $workspaceRoot
try {
    $env:JAVA_HOME = $selectedJdk
    $env:PATH = (Join-Path $selectedJdk 'bin') + [IO.Path]::PathSeparator + $previousPath
    Write-Host ('Using Java 25 JDK: ' + $selectedJdk)
    & (Join-Path $workspaceRoot 'gradlew.bat') @GradleArgs
    $gradleExitCode = $LASTEXITCODE
}
finally {
    $env:JAVA_HOME = $previousJavaHome
    $env:PATH = $previousPath
    Pop-Location
}
exit $gradleExitCode

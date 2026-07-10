# Deep equivalence harness: for each of the 18 standard benchmarks, run
# btormc on BOTH the C reference model and the freshly generated Rust model
# at kmax deep enough to cover the paper's reported bug bounds (66..1438),
# and compare (fired property index, least-k).
#
# Equivalent <=> same property index AND same least-k (or both UNSAT@kmax).
#
# Usage:  .\run_deep_equivalence.ps1 [-Kmax 1500]

[CmdletBinding()]
param([int]$Kmax = 1500)

$ErrorActionPreference = "Stop"

$HereDir  = Split-Path -Parent $MyInvocation.MyCommand.Definition
$Bins     = Join-Path $HereDir "binaries"
$CRotor   = Join-Path $HereDir "btor2-c-rotor"
$RustOut  = Join-Path $HereDir "btor2-rust-rotor"
$RotorName = if ($env:OS -eq "Windows_NT") { "rotor.exe" } else { "rotor" }
$Rotor    = Join-Path (Split-Path $HereDir) "target/release/$RotorName"
$Img      = "btormc:latest"
$CsvPath  = Join-Path $HereDir "deep_equivalence_results.csv"

if (-not (Test-Path $RustOut)) { New-Item -ItemType Directory -Path $RustOut | Out-Null }

# C-reference property names by index (identical in the Rust models now)
$PropNames = @(
  "illegal-instruction","illegal-compressed-instruction","known-instructions",
  "fetch-invalid-address","fetch-unaligned","fetch-seg-fault",
  "unknown-syscall-ID","division-by-zero","signed-division-overflow",
  "load-invalid-address","store-invalid-address",
  "compressed-load-invalid-address","compressed-store-invalid-address",
  "stack-pointer-invalid-address","load-seg-fault","store-seg-fault",
  "compressed-load-seg-fault","compressed-store-seg-fault",
  "stack-pointer-seg-fault","brk-seg-fault","openat-seg-fault",
  "read-seg-fault","write-seg-fault","bad-exit-code")

function Run-Btormc([string]$dir, [string]$file, [int]$k) {
    # returns @{Prop=<int|null>; K=<int|null>}  (null,null = UNSAT at kmax)
    $out = docker run --rm -v "${dir}:/work" $Img `
        -c "btormc -v 1 -kmax $k /work/$file 2>&1 | grep 'SATISFIABLE' | grep -v UNSAT | head -1"
    if ($out -match "bad state property (\d+) reachable at bound k = (\d+)") {
        return @{ Prop = [int]$Matches[1]; K = [int]$Matches[2] }
    }
    return @{ Prop = $null; K = $null }
}

$results = @()
$bins = Get-ChildItem -Path $Bins -Filter "*.m" | Sort-Object Name

foreach ($bin in $bins) {
    $name = $bin.BaseName
    $cFile = Join-Path $CRotor "$name.btor2"
    if (-not (Test-Path $cFile)) { Write-Host "skip ${name}: no C reference"; continue }

    Write-Host "=== $name ===" -ForegroundColor Cyan

    # 1. Generate Rust model with C-matching settings
    $rFile = Join-Path $RustOut "$name.btor2"
    & $Rotor $bin.FullName --xlen x64 --bytes-to-read 1 --heap 2048 --stack 2048 --exit-code 0 -o $rFile 2>$null
    if ($LASTEXITCODE -ne 0) { Write-Host "  rotor FAILED"; continue }

    # 2. Deep runs
    Write-Host "  btormc C...    " -NoNewline
    $c = Run-Btormc $CRotor "$name.btor2" $Kmax
    Write-Host ("prop={0} k={1}" -f ($c.Prop ?? "UNSAT"), ($c.K ?? "-"))

    Write-Host "  btormc Rust... " -NoNewline
    $r = Run-Btormc $RustOut "$name.btor2" $Kmax
    Write-Host ("prop={0} k={1}" -f ($r.Prop ?? "UNSAT"), ($r.K ?? "-"))

    $match = ($c.Prop -eq $r.Prop) -and ($c.K -eq $r.K)
    $cName = if ($null -ne $c.Prop) { $PropNames[$c.Prop] } else { "UNSAT@$Kmax" }
    $rName = if ($null -ne $r.Prop) { $PropNames[$r.Prop] } else { "UNSAT@$Kmax" }

    $row = [pscustomobject]@{
        Benchmark  = $name
        C_Property = $cName
        C_K        = $c.K
        R_Property = $rName
        R_K        = $r.K
        Equivalent = if ($match) { "YES" } else { "NO" }
    }
    $results += $row
    $results | Export-Csv -Path $CsvPath -NoTypeInformation   # incremental save

    $color = if ($match) { "Green" } else { "Red" }
    Write-Host ("  -> {0}" -f $(if ($match) { "MATCH" } else { "DIVERGE" })) -ForegroundColor $color
}

Write-Host ""
Write-Host "==================== SUMMARY ====================" -ForegroundColor Cyan
$results | Format-Table -AutoSize
$matched = ($results | Where-Object { $_.Equivalent -eq "YES" }).Count
Write-Host ("{0} / {1} benchmarks: same property at same least-k" -f $matched, $results.Count) `
    -ForegroundColor $(if ($matched -eq $results.Count) { "Green" } else { "Yellow" })
Write-Host "Results: $CsvPath"

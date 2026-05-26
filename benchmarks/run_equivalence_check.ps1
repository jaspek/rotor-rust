# Equivalence check between C Rotor and Rust Rotor on the 18 standard
# benchmarks. For each benchmark:
#   1. Use existing C Rotor BTOR2  (benchmarks/btor2-c-rotor/*.btor2)
#   2. Generate Rust Rotor BTOR2   (benchmarks/btor2-rust-rotor/*.btor2)
#   3. Run btormc on both with kmax derived from the filename
#   4. Record: did each rotor's model reach a bad state? In how many steps?
# Output: a comparison table.
#
# Settings matched to C Rotor defaults (see C-Rotor header in any .btor2):
#   bytes-to-read 1, heap 2048, stack 2048, vaddr 32-bit.
#
# Run from C:\Users\jasko\Programming\Rust\Project01\benchmarks  with:
#   .\run_equivalence_check.ps1

$ErrorActionPreference = "Stop"

$HereDir   = Split-Path -Parent $MyInvocation.MyCommand.Definition
$Bins      = Join-Path $HereDir "binaries"
$CRotor    = Join-Path $HereDir "btor2-c-rotor"
$RustOut   = Join-Path $HereDir "btor2-rust-rotor"
$Witness   = Join-Path $HereDir "btormc-witnesses"
$Rotor     = "C:\Users\jasko\Programming\Rust\Project01\target\release\rotor.exe"
$BtormcImg = "btormc:latest"

if (Test-Path $RustOut) { Remove-Item $RustOut -Recurse -Force }
New-Item -ItemType Directory -Path $RustOut | Out-Null
if (Test-Path $Witness) { Remove-Item $Witness -Recurse -Force }
New-Item -ItemType Directory -Path $Witness | Out-Null

# ---------------------------------------------------------------------------
# Step 1 — Generate Rust Rotor outputs (matching C Rotor settings)
# ---------------------------------------------------------------------------
Write-Host "==> Step 1: Generating Rust Rotor outputs" -ForegroundColor Cyan

$bins = Get-ChildItem -Path $Bins -Filter "*.m"
foreach ($bin in $bins) {
    $name = $bin.BaseName
    $outFile = Join-Path $RustOut "$name.btor2"
    Write-Host "  $name ... " -NoNewline
    & $Rotor $bin.FullName --xlen x64 --bytes-to-read 1 --heap 2048 --stack 2048 -o $outFile 2>$null
    if ($LASTEXITCODE -eq 0) {
        $size = (Get-Item $outFile).Length
        Write-Host ("OK  ({0:N0} bytes)" -f $size) -ForegroundColor Green
    } else {
        Write-Host "FAIL (exit $LASTEXITCODE)" -ForegroundColor Red
    }
}

# ---------------------------------------------------------------------------
# Step 2 — Run btormc on both versions of each benchmark
# ---------------------------------------------------------------------------
Write-Host ""
Write-Host "==> Step 2: Running btormc on both rotors' outputs" -ForegroundColor Cyan

function Get-Kmax($name) {
    # name = "simple-assignment-1-35" -> 35
    # name = "recursive-fibonacci-1-10" -> 10
    if ($name -match "-(\d+)$") {
        return [int]$matches[1]
    }
    return 35  # default
}

$results = @()

foreach ($bin in $bins) {
    $name  = $bin.BaseName
    $kmax  = Get-Kmax $name
    $cFile = Join-Path $CRotor   "$name.btor2"
    $rFile = Join-Path $RustOut  "$name.btor2"

    if (-not (Test-Path $cFile)) {
        Write-Host "  ${name}: C Rotor file missing, skip" -ForegroundColor Yellow
        continue
    }
    if (-not (Test-Path $rFile)) {
        Write-Host "  ${name}: Rust Rotor file missing, skip" -ForegroundColor Yellow
        continue
    }

    Write-Host "  $name (kmax=$kmax)" -ForegroundColor White

    # C Rotor: btormc inside docker, capture stdout+stderr to a file inside the container
    $cLog = Join-Path $Witness "$name.c.txt"
    docker run --rm -v "${CRotor}:/work" -v "${Witness}:/log" $BtormcImg `
        -c "btormc -kmax $kmax /work/$name.btor2 > /log/$name.c.txt 2>&1 ; echo EXITCODE=`$? >> /log/$name.c.txt" | Out-Null

    # Rust Rotor: same idea
    $rLog = Join-Path $Witness "$name.rust.txt"
    docker run --rm -v "${RustOut}:/work" -v "${Witness}:/log" $BtormcImg `
        -c "btormc -kmax $kmax /work/$name.btor2 > /log/$name.rust.txt 2>&1 ; echo EXITCODE=`$? >> /log/$name.rust.txt" | Out-Null

    # Parse the outputs.
    # btormc prints "sat\nb<i>\n<step>\n..." when it finds a bad state,
    # and "unsat" or "unknown" otherwise.
    function Parse-BtormcLog($path) {
        if (-not (Test-Path $path)) { return @{ Result = "no-output"; Steps = $null } }
        $text = Get-Content $path -Raw
        if (-not $text) { return @{ Result = "no-output"; Steps = $null } }
        # btormc emits "sat" (lowercase) and bad-state witness frames @0 @1 ...
        # OR remains silent on unsat (default verbosity)
        if ($text -match "(?m)^sat\b") {
            $steps = $null
            $lines = $text -split "`r?`n"
            $sawSat = $false
            foreach ($l in $lines) {
                if ($sawSat -and $l -match "^@(\d+)") {
                    $steps = [int]$matches[1]
                    break
                }
                if ($l -match "^sat") { $sawSat = $true }
            }
            return @{ Result = "sat"; Steps = $steps }
        } elseif ($text -match "(?m)^unsat\b") {
            return @{ Result = "unsat"; Steps = $null }
        } elseif ($text -match "EXITCODE=0") {
            # Silent exit-0 means btormc completed without finding a bad state
            # within kmax. We treat that as 'no-bug' (effectively unsat at kmax).
            return @{ Result = "no-bug"; Steps = $null }
        } elseif ($text -match "EXITCODE=") {
            return @{ Result = "error"; Steps = $null }
        } else {
            return @{ Result = "unknown"; Steps = $null }
        }
    }

    $cRes = Parse-BtormcLog $cLog
    $rRes = Parse-BtormcLog $rLog

    $match = ($cRes.Result -eq $rRes.Result)

    $results += [pscustomobject]@{
        Benchmark   = $name
        Kmax        = $kmax
        C_Result    = $cRes.Result
        C_Steps     = $cRes.Steps
        Rust_Result = $rRes.Result
        Rust_Steps  = $rRes.Steps
        Equivalent  = if ($match) { "YES" } else { "NO" }
    }

    $color = if ($match) { "Green" } else { "Red" }
    Write-Host ("    C: {0,-8} (step {1,-4})  Rust: {2,-8} (step {3,-4})  ->  {4}" -f
        $cRes.Result, $cRes.Steps, $rRes.Result, $rRes.Steps,
        $(if ($match) { "MATCH" } else { "DIVERGE" })) -ForegroundColor $color
}

# ---------------------------------------------------------------------------
# Step 3 — Summary table
# ---------------------------------------------------------------------------
Write-Host ""
Write-Host "==> Summary" -ForegroundColor Cyan
$results | Format-Table -AutoSize

# Quick stats
$total = $results.Count
$matched = ($results | Where-Object { $_.Equivalent -eq "YES" }).Count
Write-Host ""
Write-Host ("$matched / $total benchmarks gave the same btormc verdict") -ForegroundColor Green

# Also save results to CSV for the slide
$csvPath = Join-Path $HereDir "equivalence_results.csv"
$results | Export-Csv -Path $csvPath -NoTypeInformation
Write-Host "Wrote results to: $csvPath" -ForegroundColor Cyan

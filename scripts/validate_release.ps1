# Release validation script for Uzima-Contracts (PowerShell version)
# Usage: .\scripts\validate_release.ps1 -Version "1.2.0" [-DryRun] [-SkipTests] [-SkipSecurity] [-Strict]

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,
    
    [switch]$DryRun,
    [switch]$SkipTests,
    [switch]$SkipSecurity,
    [switch]$Strict
)

# Configuration
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir

# Validation results
$ValidationErrors = 0
$ValidationWarnings = 0

# Logging functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
    $script:ValidationWarnings++
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
    $script:ValidationErrors++
}

# Validation functions
function Test-VersionFormat {
    param([string]$Version)
    
    Write-Info "Validating version format: $Version"
    
    if ($Version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$') {
        Write-Error "Invalid version format: $Version"
        Write-Error "Expected format: X.Y.Z or X.Y.Z-PRERELEASE"
        return $false
    }
    
    Write-Success "Version format is valid"
    return $true
}

function Test-GitState {
    Write-Info "Validating git state..."
    
    # Check if we're in a git repository
    try {
        git rev-parse --git-dir 2>$null | Out-Null
    } catch {
        Write-Error "Not in a git repository"
        return $false
    }
    
    # Check if working directory is clean
    $gitStatus = git status --porcelain
    if ($gitStatus) {
        Write-Error "Working directory is not clean"
        Write-Host $gitStatus
        return $false
    }
    
    # Check if tag already exists
    $tag = "v$Version"
    $tagCheck = git rev-parse $tag 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Error "Tag $tag already exists"
        return $false
    }
    # Tag doesn't exist, which is good
    
    Write-Success "Git state is valid"
    return $true
}

function Test-Dependencies {
    Write-Info "Validating dependencies..."
    
    # Check required commands
    $requiredCommands = @("git", "cargo", "soroban")
    foreach ($cmd in $requiredCommands) {
        try {
            Get-Command $cmd -ErrorAction Stop | Out-Null
        } catch {
            Write-Error "Required command not found: $cmd"
            return $false
        }
    }
    
    # Check Rust version
    try {
        $rustVersion = rustc --version | ForEach-Object { ($_ -split ' ')[1] }
        Write-Info "Rust version: $rustVersion"
    } catch {
        Write-Warning "Could not determine Rust version"
    }
    
    # Check Soroban version
    try {
        $sorobanVersion = soroban --version | ForEach-Object { ($_ -split ' ')[1] }
        Write-Info "Soroban version: $sorobanVersion"
    } catch {
        Write-Warning "Could not determine Soroban version"
    }
    
    Write-Success "Dependencies are valid"
    return $true
}

function Test-Changelog {
    Write-Info "Validating changelog..."
    
    $changelogFile = Join-Path $ProjectRoot "CHANGELOG.md"
    
    if (-not (Test-Path $changelogFile)) {
        Write-Error "CHANGELOG.md not found"
        return $false
    }
    
    # Check if version entry exists
    $changelogContent = Get-Content $changelogFile -Raw
    if ($changelogContent -notmatch "## \[$Version\]") {
        Write-Error "Changelog entry for version $Version not found"
        return $false
    }
    
    Write-Success "Changelog is valid"
    return $true
}

function Test-CodeQuality {
    Write-Info "Validating code quality..."
    
    Push-Location $ProjectRoot
    
    try {
        # Format check
        $formatResult = cargo fmt --all -- --check 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Code formatting check failed"
            return $false
        }
        
        # Clippy check
        $clippyResult = cargo clippy --all-targets --all-features -- -D warnings 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Clippy check failed"
            return $false
        }
        
        Write-Success "Code quality is valid"
        return $true
    } finally {
        Pop-Location
    }
}

function Test-Tests {
    if ($SkipTests) {
        Write-Warning "Skipping tests (SkipTests specified)"
        return $true
    }
    
    Write-Info "Running tests..."
    
    Push-Location $ProjectRoot
    
    try {
        # Unit tests
        $unitResult = cargo test --lib 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Unit tests failed"
            return $false
        }
        
        # Integration tests
        $integrationResult = cargo test --test integration 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Integration tests failed"
            return $false
        }
        
        Write-Success "All tests passed"
        return $true
    } finally {
        Pop-Location
    }
}

function Test-Build {
    Write-Info "Validating build..."
    
    Push-Location $ProjectRoot
    
    try {
        # Clean build
        if (Test-Path "makefile") {
            # Use make if available
            make clean 2>$null
        } else {
            cargo clean 2>$null
        }
        
        # Build optimized contracts
        if (Test-Path "makefile") {
            $buildResult = make build-opt 2>&1
        } else {
            $buildResult = cargo build --target wasm32-unknown-unknown --release 2>&1
        }
        
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Build failed"
            return $false
        }
        
        Write-Success "Build is valid"
        return $true
    } finally {
        Pop-Location
    }
}

function Test-Security {
    if ($SkipSecurity) {
        Write-Warning "Skipping security checks (SkipSecurity specified)"
        return $true
    }
    
    Write-Info "Running security validation..."
    
    Push-Location $ProjectRoot
    
    try {
        # Security audit
        try {
            Get-Command cargo-audit -ErrorAction Stop | Out-Null
            $auditResult = cargo audit 2>&1
            if ($LASTEXITCODE -ne 0) {
                Write-Error "Security audit failed"
                return $false
            }
        } catch {
            Write-Warning "cargo-audit not installed, skipping security audit"
        }
        
        # Security-focused clippy
        $securityClippyResult = cargo clippy --all-targets --all-features -- -W clippy::indexing_slicing -W clippy::panic -W clippy::unwrap_used 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Security-focused clippy failed"
            return $false
        }
        
        Write-Success "Security validation passed"
        return $true
    } finally {
        Pop-Location
    }
}

function Test-Versions {
    Write-Info "Validating version consistency..."
    
    Push-Location $ProjectRoot
    
    try {
        # Check workspace version
        $cargoToml = Get-Content "Cargo.toml" -Raw
        if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
            $workspaceVersion = $matches[1]
            if ($workspaceVersion -ne $Version) {
                Write-Error "Workspace version mismatch: expected $Version, found $workspaceVersion"
                return $false
            }
        }
        
        # Check contract versions
        $contractDirs = Get-ChildItem "contracts" -Directory
        foreach ($contractDir in $contractDirs) {
            $contractCargoToml = Join-Path $contractDir.FullName "Cargo.toml"
            if (Test-Path $contractCargoToml) {
                $contractContent = Get-Content $contractCargoToml -Raw
                if ($contractContent -match 'version\s*=\s*"([^"]+)"') {
                    $contractVersion = $matches[1]
                    if ($contractVersion -ne $Version) {
                        Write-Error "Contract $($contractDir.Name) version mismatch: expected $Version, found $contractVersion"
                        return $false
                    }
                }
            }
        }
        
        Write-Success "Version consistency validated"
        return $true
    } finally {
        Pop-Location
    }
}

function Test-Documentation {
    Write-Info "Validating documentation..."
    
    # Check README exists
    $readmeFile = Join-Path $ProjectRoot "README.md"
    if (-not (Test-Path $readmeFile)) {
        Write-Error "README.md not found"
        return $false
    }
    
    # Check versioning documentation
    $versioningDocs = @(
        "docs/VERSIONING_STRATEGY.md",
        "docs/RELEASE_PROCESS.md", 
        "docs/CHANGELOG_FORMAT.md"
    )
    
    foreach ($doc in $versioningDocs) {
        $docPath = Join-Path $ProjectRoot $doc
        if (-not (Test-Path $docPath)) {
            Write-Error "Documentation file not found: $doc"
            return $false
        }
    }
    
    Write-Success "Documentation is valid"
    return $true
}

function Test-Artifacts {
    Write-Info "Validating build artifacts..."
    
    Push-Location $ProjectRoot
    
    try {
        # Create dist directory
        if (Test-Path "makefile") {
            make dist 2>$null
        } else {
            # Manual dist creation
            if (-not (Test-Path "dist")) {
                New-Item -ItemType Directory -Path "dist" | Out-Null
            }
            
            # Copy WASM files
            $contractDirs = Get-ChildItem "contracts" -Directory
            foreach ($contractDir in $contractDirs) {
                $wasmFile = Join-Path $contractDir.FullName "target/wasm32-unknown-unknown/release/$($contractDir.Name).wasm"
                if (Test-Path $wasmFile) {
                    Copy-Item $wasmFile "dist/$($contractDir.Name).wasm"
                }
            }
        }
        
        # Check if WASM files exist
        $wasmFiles = Get-ChildItem "dist/*.wasm" -ErrorAction SilentlyContinue
        if ($wasmFiles.Count -eq 0) {
            Write-Error "No WASM files found in dist/"
            return $false
        }
        
        # Validate each WASM file
        foreach ($wasmFile in $wasmFiles) {
            if ($wasmFile.Length -eq 0) {
                Write-Error "WASM file is empty: $($wasmFile.Name)"
                return $false
            }
            
            if ($wasmFile.Length -gt 65536) {
                Write-Warning "WASM file exceeds size limit: $($wasmFile.Name) ($($wasmFile.Length) bytes)"
                if ($Strict) {
                    return $false
                }
            }
        }
        
        Write-Success "Build artifacts are valid"
        return $true
    } finally {
        Pop-Location
    }
}

# Main validation function
function Start-Validation {
    param([string]$Version)
    
    Write-Info "Starting comprehensive release validation for v$Version..."
    Write-Host ""
    
    # Core validations
    if (-not (Test-VersionFormat $Version)) { return $false }
    if (-not (Test-GitState)) { return $false }
    if (-not (Test-Dependencies)) { return $false }
    if (-not (Test-Changelog)) { return $false }
    if (-not (Test-Versions)) { return $false }
    if (-not (Test-Documentation)) { return $false }
    if (-not (Test-CodeQuality)) { return $false }
    
    # Build and tests
    if (-not (Test-Build)) { return $false }
    if (-not (Test-Tests)) { return $false }
    
    # Security and artifacts
    if (-not (Test-Security)) { return $false }
    if (-not (Test-Artifacts)) { return $false }
    
    Write-Host ""
    Write-Info "Validation completed"
    Write-Host ""
    
    # Summary
    if ($ValidationErrors -gt 0) {
        Write-Error "Validation failed with $ValidationErrors error(s)"
        return $false
    }
    
    if ($ValidationWarnings -gt 0) {
        Write-Warning "Validation completed with $ValidationWarnings warning(s)"
    } else {
        Write-Success "All validations passed successfully! 🎉"
    }
    
    return $true
}

# Main execution
try {
    $success = Start-Validation -Version $Version
    
    if (-not $success) {
        exit 1
    }
} catch {
    Write-Error "Validation script failed: $($_.Exception.Message)"
    exit 1
}

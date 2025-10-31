# The Agency - Windows Service Installation Script
# Run as Administrator

param(
    [string]$ServiceName = "AgencyService",
    [string]$DisplayName = "The Agency AI Platform",
    [string]$BinaryPath = "C:\Program Files\TheAgency\agency-service.exe",
    [string]$ConfigPath = "C:\ProgramData\Agency\config.toml"
)

Write-Host "The Agency - Windows Service Installation" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host ""

# Check if running as administrator
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "This script must be run as Administrator"
    exit 1
}

# Create directories
Write-Host "Creating directories..."
$dirs = @(
    "C:\Program Files\TheAgency",
    "C:\ProgramData\Agency",
    "C:\ProgramData\Agency\data",
    "C:\ProgramData\Agency\logs"
)

foreach ($dir in $dirs) {
    if (-not (Test-Path $dir)) {
        New-Item -Path $dir -ItemType Directory -Force | Out-Null
        Write-Host "  Created: $dir" -ForegroundColor Gray
    }
}

# Copy binary
Write-Host ""
Write-Host "Copying binary..."
if (Test-Path ".\target\release\agency-service.exe") {
    Copy-Item ".\target\release\agency-service.exe" -Destination $BinaryPath -Force
    Write-Host "  Binary copied to: $BinaryPath" -ForegroundColor Gray
} else {
    Write-Error "Binary not found. Please build the project first: cargo build --release --bin agency-service"
    exit 1
}

# Copy configuration
Write-Host ""
Write-Host "Copying configuration..."
if (Test-Path ".\config.example.toml") {
    if (-not (Test-Path $ConfigPath)) {
        Copy-Item ".\config.example.toml" -Destination $ConfigPath -Force
        Write-Host "  Config copied to: $ConfigPath" -ForegroundColor Gray
    } else {
        Write-Host "  Config already exists at: $ConfigPath" -ForegroundColor Yellow
    }
}

# Check if service already exists
Write-Host ""
Write-Host "Checking existing service..."
$existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue

if ($existingService) {
    Write-Host "  Service already exists. Stopping and removing..." -ForegroundColor Yellow
    Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
    sc.exe delete $ServiceName
    Start-Sleep -Seconds 2
}

# Create service
Write-Host ""
Write-Host "Creating Windows service..."
sc.exe create $ServiceName `
    binPath= $BinaryPath `
    DisplayName= $DisplayName `
    start= auto `
    obj= "LocalSystem"

if ($LASTEXITCODE -eq 0) {
    Write-Host "  Service created successfully" -ForegroundColor Green
    
    # Set service description
    sc.exe description $ServiceName "AI Agent Platform with REST API, workflow orchestration, and saga pattern support"
    
    # Configure service recovery
    sc.exe failure $ServiceName reset= 86400 actions= restart/60000/restart/60000/restart/60000
    
    Write-Host ""
    Write-Host "Service installed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "To start the service, run:" -ForegroundColor Cyan
    Write-Host "  Start-Service $ServiceName" -ForegroundColor White
    Write-Host ""
    Write-Host "To check service status:" -ForegroundColor Cyan
    Write-Host "  Get-Service $ServiceName" -ForegroundColor White
    Write-Host ""
    Write-Host "Configuration file location:" -ForegroundColor Cyan
    Write-Host "  $ConfigPath" -ForegroundColor White
    Write-Host ""
} else {
    Write-Error "Failed to create service"
    exit 1
}

# Prompt to start service
$response = Read-Host "Do you want to start the service now? (Y/N)"
if ($response -eq 'Y' -or $response -eq 'y') {
    Write-Host ""
    Write-Host "Starting service..."
    Start-Service -Name $ServiceName
    Start-Sleep -Seconds 2
    
    $service = Get-Service -Name $ServiceName
    if ($service.Status -eq 'Running') {
        Write-Host "Service started successfully!" -ForegroundColor Green
        Write-Host ""
        Write-Host "API available at: http://localhost:8080" -ForegroundColor Cyan
        Write-Host "Health check: http://localhost:8080/health" -ForegroundColor Cyan
    } else {
        Write-Error "Service failed to start. Check Event Viewer for details."
    }
}

Write-Host ""
Write-Host "Installation complete!" -ForegroundColor Green

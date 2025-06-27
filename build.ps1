$tempDir = [System.IO.Path]::GetTempPath() + "temporary_files"

# If the directory doesn't exist, create it
if (-Not (Test-Path -Path $tempDir)) {
    New-Item -Path $tempDir -ItemType Directory
}

# Change to the temporary directory
Set-Location -Path $tempDir

# Run the Cargo build command
Write-Host "Running cargo build --release..."
cargo build --release

# Run the Python setup.py install command
Write-Host "Running python setup.py install..."
python setup.py install

# Clean up the temporary directory (optional)
Write-Host "Cleaning up the temporary directory..."
Remove-Item -Path $tempDir -Recurse -Force

Write-Host "Process completed!"

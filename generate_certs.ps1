# MQTT TLS Certificate Generation Script
# Generate CA, Server and Client certificates using OpenSSL

$ErrorActionPreference = "Stop"

Write-Host "Generating MQTT TLS certificates..." -ForegroundColor Green

# Check if OpenSSL is available
$openssl = Get-Command openssl -ErrorAction SilentlyContinue
if (-not $openssl) {
    Write-Host "Error: OpenSSL not found" -ForegroundColor Red
    Write-Host "Please install OpenSSL or use Git for Windows bundled OpenSSL" -ForegroundColor Yellow
    Write-Host "Download: https://slproweb.com/products/Win32OpenSSL.html" -ForegroundColor Yellow
    exit 1
}

$certDir = "certs"
if (-not (Test-Path $certDir)) {
    New-Item -ItemType Directory -Path $certDir | Out-Null
}

Set-Location $certDir

Write-Host "`nStep 1/4: Generating CA private key..." -ForegroundColor Cyan
openssl genrsa -out ca.key 2048
if ($LASTEXITCODE -ne 0) { throw "Failed to generate CA private key" }

Write-Host "Step 2/4: Generating CA certificate..." -ForegroundColor Cyan
openssl req -new -x509 -days 3650 -key ca.key -out ca.cert.pem -subj '/C=CN/ST=Beijing/L=Beijing/O=MQTT Broker/CN=MQTT CA'
if ($LASTEXITCODE -ne 0) { throw "Failed to generate CA certificate" }

Write-Host "Step 3/4: Generating server private key and certificate..." -ForegroundColor Cyan
openssl genrsa -out server.key 2048
if ($LASTEXITCODE -ne 0) { throw "Failed to generate server private key" }

openssl req -new -key server.key -out server.csr -subj '/C=CN/ST=Beijing/L=Beijing/O=MQTT Broker/CN=localhost'
if ($LASTEXITCODE -ne 0) { throw "Failed to generate server certificate request" }

# Create server certificate extension file
$extContent = @"
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req

[v3_req]
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
DNS.2 = *.localhost
IP.1 = 127.0.0.1
IP.2 = ::1
"@
$extContent | Out-File -FilePath server.ext -Encoding ASCII -NoNewline

openssl x509 -req -in server.csr -CA ca.cert.pem -CAkey ca.key -CAcreateserial -out server.cert.pem -days 3650 -extensions v3_req -extfile server.ext
if ($LASTEXITCODE -ne 0) { throw "Failed to generate server certificate" }

Write-Host "Step 4/4: Generating client private key and certificate..." -ForegroundColor Cyan
openssl genrsa -out client.key 2048
if ($LASTEXITCODE -ne 0) { throw "Failed to generate client private key" }

openssl req -new -key client.key -out client.csr -subj '/C=CN/ST=Beijing/L=Beijing/O=MQTT Client/CN=mqtt-client'
if ($LASTEXITCODE -ne 0) { throw "Failed to generate client certificate request" }

openssl x509 -req -in client.csr -CA ca.cert.pem -CAkey ca.key -CAcreateserial -out client.cert.pem -days 3650
if ($LASTEXITCODE -ne 0) { throw "Failed to generate client certificate" }

# Clean up temporary files
Remove-Item server.csr, client.csr, server.ext, ca.srl -ErrorAction SilentlyContinue

Set-Location ..

Write-Host "`nCertificate generation completed!" -ForegroundColor Green
Write-Host "`nGenerated files:" -ForegroundColor Cyan
Write-Host "  certs/ca.cert.pem      - CA certificate" -ForegroundColor White
Write-Host "  certs/ca.key           - CA private key" -ForegroundColor White
Write-Host "  certs/server.cert.pem  - Server certificate" -ForegroundColor White
Write-Host "  certs/server.key       - Server private key" -ForegroundColor White
Write-Host "  certs/client.cert.pem  - Client certificate" -ForegroundColor White
Write-Host "  certs/client.key       - Client private key" -ForegroundColor White
Write-Host "`nNote: These are self-signed certificates for development and testing only!" -ForegroundColor Yellow

#!/bin/sh

echo "Checking for certificates..."
if [ ! -f certs/cert.pem ]; then
    echo "Certificates not found, generating self-signed certificate for $TLS_HOST..."
    mkdir -p certs
    openssl req -x509 -newkey rsa:4096 -keyout certs/key.pem -out certs/cert.pem -days 365 -nodes -subj "/CN=$TLS_HOST"
    echo "Self-signed certificate generated."
else
    echo "Certificates found."
fi

echo "Extracting TLS host from certificate..."
TLS_HOST=$(openssl x509 -in certs/cert.pem -noout -subject | sed "s/.*CN=\([^,]*\).*/\1/")
echo "TLS host extracted: $TLS_HOST"

echo "Starting static-web-server..."
exec static-web-server --host 0.0.0.0 --port 443 --https-redirect --https-redirect-host "$TLS_HOST" --https-redirect-from-hosts "$TLS_HOST" -w sws.toml --http2 --http2-tls-cert IGNORED_ANYWAYS --http2-tls-key IGNORED_ANYWAYS
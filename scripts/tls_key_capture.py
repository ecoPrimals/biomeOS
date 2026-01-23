#!/usr/bin/env python3
"""
TLS 1.3 Key Capture Script
Captures traffic secrets from TLS connections for comparison with BearDog output.
"""

import ssl
import socket
import sys
from pathlib import Path

def capture_tls_keys(hostname, port=443):
    """Connect to a host and capture TLS traffic secrets."""
    
    # Create key log file
    keylog_file = Path("/tmp/python-tls-keys.log")
    keylog_file.unlink(missing_ok=True)
    
    print(f"🔍 Connecting to {hostname}:{port} with TLS 1.3...")
    print(f"📝 Key log: {keylog_file}")
    print()
    
    # Create SSL context
    context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
    context.minimum_version = ssl.TLSVersion.TLSv1_3
    context.maximum_version = ssl.TLSVersion.TLSv1_3
    context.check_hostname = True
    context.verify_mode = ssl.CERT_REQUIRED
    context.load_default_certs()
    
    # Enable key logging
    context.keylog_filename = str(keylog_file)
    
    # Connect
    with socket.create_connection((hostname, port), timeout=10) as sock:
        with context.wrap_socket(sock, server_hostname=hostname) as ssock:
            print(f"✅ Connected!")
            print(f"   Protocol: {ssock.version()}")
            print(f"   Cipher: {ssock.cipher()[0]}")
            print()
            
            # Send HTTP request
            request = f"GET / HTTP/1.1\r\nHost: {hostname}\r\nConnection: close\r\n\r\n"
            ssock.sendall(request.encode())
            
            # Read a bit of response
            response = ssock.recv(1024)
            print(f"✅ Received {len(response)} bytes")
            status_line = response.split(b'\r\n')[0].decode()
            print(f"   Status: {status_line}")
            print()
    
    # Read and display keys
    if keylog_file.exists():
        print("📊 Captured TLS Keys:")
        print("=" * 80)
        keys = keylog_file.read_text().strip().split('\n')
        for line in keys:
            if line.startswith('CLIENT_TRAFFIC_SECRET_0'):
                parts = line.split()
                print("\n🔑 CLIENT_TRAFFIC_SECRET_0:")
                print(f"   Client Random: {parts[1]}")
                print(f"   Secret: {parts[2]}")
            elif line.startswith('SERVER_TRAFFIC_SECRET_0'):
                parts = line.split()
                print("\n🔑 SERVER_TRAFFIC_SECRET_0:")
                print(f"   Client Random: {parts[1]}")
                print(f"   Secret: {parts[2]}")
            elif line.startswith('CLIENT_HANDSHAKE_TRAFFIC_SECRET'):
                parts = line.split()
                print("\n🔑 CLIENT_HANDSHAKE_TRAFFIC_SECRET:")
                print(f"   Client Random: {parts[1]}")
                print(f"   Secret: {parts[2]}")
            elif line.startswith('SERVER_HANDSHAKE_TRAFFIC_SECRET'):
                parts = line.split()
                print("\n🔑 SERVER_HANDSHAKE_TRAFFIC_SECRET:")
                print(f"   Client Random: {parts[1]}")
                print(f"   Secret: {parts[2]}")
        print("=" * 80)
        print()
        print(f"📝 Full key log saved to: {keylog_file}")
        return keylog_file
    else:
        print("❌ No key log file created!")
        return None

if __name__ == "__main__":
    hostname = sys.argv[1] if len(sys.argv) > 1 else "example.com"
    try:
        capture_tls_keys(hostname)
    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)


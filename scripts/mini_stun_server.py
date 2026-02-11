#!/usr/bin/env python3
"""
Minimal STUN Server for biomeOS Testing
========================================

TEMPORARY: Testing bridge while waiting for:
1. coturn installation (production)
2. Songbird pure Rust STUN server (sovereign)

This is a minimal RFC 5389 STUN server that handles Binding Requests
and returns the client's public IP:port as MAPPED-ADDRESS.

Usage:
    python3 mini_stun_server.py [port]
    
    Default port: 3478 (standard STUN port)

Test with Songbird:
    echo '{"jsonrpc":"2.0","method":"stun.get_public_address",
      "params":{"server":"127.0.0.1:3478"},"id":1}' | 
      nc -U /run/user/1000/biomeos/songbird-$FAMILY_ID.sock

License: AGPL-3.0-only (biomeOS)
"""

import socket
import struct
import sys
import os
from typing import Tuple, Optional

# STUN Message Types
STUN_BINDING_REQUEST = 0x0001
STUN_BINDING_RESPONSE = 0x0101

# STUN Attribute Types
ATTR_MAPPED_ADDRESS = 0x0001
ATTR_XOR_MAPPED_ADDRESS = 0x0020
ATTR_SOFTWARE = 0x8022

# STUN Magic Cookie (RFC 5389)
MAGIC_COOKIE = 0x2112A442


def parse_stun_request(data: bytes) -> Optional[Tuple[int, bytes]]:
    """Parse STUN request, return (message_type, transaction_id) or None"""
    if len(data) < 20:
        return None
    
    # STUN Header: Type (2) + Length (2) + Magic Cookie (4) + Transaction ID (12)
    msg_type, msg_len, magic = struct.unpack(">HHI", data[:8])
    
    if magic != MAGIC_COOKIE:
        print(f"  Invalid magic cookie: {magic:#x}")
        return None
    
    transaction_id = data[8:20]
    return msg_type, transaction_id


def create_mapped_address_attr(addr: Tuple[str, int]) -> bytes:
    """Create MAPPED-ADDRESS attribute (0x0001)"""
    ip, port = addr
    
    # Family: 0x01 = IPv4
    family = 0x01
    
    # Pack IP address
    ip_parts = [int(p) for p in ip.split('.')]
    
    # Attribute value: Reserved (1) + Family (1) + Port (2) + IP (4)
    value = struct.pack(">xBH4B", family, port, *ip_parts)
    
    # Attribute: Type (2) + Length (2) + Value
    return struct.pack(">HH", ATTR_MAPPED_ADDRESS, len(value)) + value


def create_xor_mapped_address_attr(addr: Tuple[str, int], transaction_id: bytes) -> bytes:
    """Create XOR-MAPPED-ADDRESS attribute (0x0020)"""
    ip, port = addr
    
    # XOR port with top 16 bits of magic cookie
    xor_port = port ^ (MAGIC_COOKIE >> 16)
    
    # XOR IP with magic cookie
    ip_int = sum(int(p) << (24 - 8*i) for i, p in enumerate(ip.split('.')))
    xor_ip = ip_int ^ MAGIC_COOKIE
    
    # Family: 0x01 = IPv4
    family = 0x01
    
    # Attribute value
    value = struct.pack(">xBHI", family, xor_port, xor_ip)
    
    # Attribute: Type (2) + Length (2) + Value
    return struct.pack(">HH", ATTR_XOR_MAPPED_ADDRESS, len(value)) + value


def create_software_attr(software: str) -> bytes:
    """Create SOFTWARE attribute (0x8022)"""
    value = software.encode('utf-8')
    # Pad to 4-byte boundary
    padding = (4 - len(value) % 4) % 4
    value += b'\x00' * padding
    
    return struct.pack(">HH", ATTR_SOFTWARE, len(software)) + value


def create_binding_response(transaction_id: bytes, client_addr: Tuple[str, int]) -> bytes:
    """Create STUN Binding Response"""
    
    # Build attributes
    attrs = b''
    attrs += create_mapped_address_attr(client_addr)
    attrs += create_xor_mapped_address_attr(client_addr, transaction_id)
    attrs += create_software_attr("biomeOS-mini-stun/1.0")
    
    # STUN Header
    header = struct.pack(">HHI", STUN_BINDING_RESPONSE, len(attrs), MAGIC_COOKIE)
    header += transaction_id
    
    return header + attrs


def run_server(port: int = 3478):
    """Run minimal STUN server"""
    
    print("╔═══════════════════════════════════════════════════════════════════════════╗")
    print("║              biomeOS Mini STUN Server (Testing Only)                     ║")
    print("╠═══════════════════════════════════════════════════════════════════════════╣")
    print(f"║  Listening on: 0.0.0.0:{port}                                             ║")
    print("║  Press Ctrl+C to stop                                                    ║")
    print("╚═══════════════════════════════════════════════════════════════════════════╝")
    print()
    
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(('0.0.0.0', port))
    
    print(f"STUN server listening on port {port}...")
    print()
    
    requests_handled = 0
    
    try:
        while True:
            data, client_addr = sock.recvfrom(1500)
            
            print(f"[{requests_handled + 1}] Request from {client_addr[0]}:{client_addr[1]}")
            
            parsed = parse_stun_request(data)
            if parsed is None:
                print("  → Invalid STUN request, ignoring")
                continue
            
            msg_type, transaction_id = parsed
            
            if msg_type == STUN_BINDING_REQUEST:
                print(f"  → Binding Request (txn: {transaction_id.hex()[:8]}...)")
                
                # Create and send response
                response = create_binding_response(transaction_id, client_addr)
                sock.sendto(response, client_addr)
                
                print(f"  ← Binding Response: MAPPED-ADDRESS = {client_addr[0]}:{client_addr[1]}")
                requests_handled += 1
            else:
                print(f"  → Unknown message type: {msg_type:#x}")
            
            print()
            
    except KeyboardInterrupt:
        print()
        print(f"Shutting down. Handled {requests_handled} requests.")
    finally:
        sock.close()


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 3478
    run_server(port)

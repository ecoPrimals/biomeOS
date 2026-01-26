#!/usr/bin/env python3
"""
HKDF-Expand-Label Validation Test
==================================

Validates BearDog's HKDF-Expand-Label implementation by testing with
known CLIENT_TRAFFIC_SECRET_0 and verifying derived keys match expected outputs.

Based on RFC 8446 Section 7.1: HKDF-Expand-Label
"""

import hmac
import hashlib
import binascii

def hkdf_expand_label(secret, label, context, length):
    """
    HKDF-Expand-Label(Secret, Label, Context, Length) as defined in RFC 8446
    
    Args:
        secret: bytes - The secret to expand
        label: str - The label (will be prefixed with "tls13 ")
        context: bytes - The context (usually empty for key/IV derivation)
        length: int - Output length in bytes
    
    Returns:
        bytes - Expanded key material of specified length
    """
    # Construct HkdfLabel structure:
    # struct {
    #     uint16 length = Length;
    #     opaque label<7..255> = "tls13 " + Label;
    #     opaque context<0..255> = Context;
    # } HkdfLabel;
    
    full_label = b"tls13 " + label.encode('ascii')
    
    hkdf_label = b''
    # Length (2 bytes, big-endian)
    hkdf_label += length.to_bytes(2, 'big')
    # Label length + label
    hkdf_label += len(full_label).to_bytes(1, 'big') + full_label
    # Context length + context
    hkdf_label += len(context).to_bytes(1, 'big') + context
    
    # HKDF-Expand using SHA-256 (for cipher suite 0x1301)
    return hkdf_expand(secret, hkdf_label, length, hashlib.sha256)

def hkdf_expand(prk, info, length, hash_func):
    """
    HKDF-Expand as defined in RFC 5869
    
    Args:
        prk: bytes - Pseudorandom key (from HKDF-Extract)
        info: bytes - Optional context/application-specific info
        length: int - Length of output keying material in bytes
        hash_func: hash function to use (e.g., hashlib.sha256)
    
    Returns:
        bytes - Output keying material of specified length
    """
    hash_len = hash_func().digest_size
    n = (length + hash_len - 1) // hash_len  # Ceiling division
    
    t = b''
    okm = b''
    for i in range(1, n + 1):
        t = hmac.new(prk, t + info + bytes([i]), hash_func).digest()
        okm += t
    
    return okm[:length]

def main():
    print("═" * 79)
    print("🔬 HKDF-Expand-Label Validation Test")
    print("═" * 79)
    print()
    
    # Known CLIENT_TRAFFIC_SECRET_0 from our logs
    client_traffic_secret_0 = binascii.unhexlify(
        "2c6504277fb08472812caf1c34f4bbc8118223c96f7e9b28ed0aae867fa06720"
    )
    
    print("Input:")
    print("─" * 79)
    print(f"CLIENT_TRAFFIC_SECRET_0 (32 bytes):")
    print(f"  {binascii.hexlify(client_traffic_secret_0).decode()}")
    print()
    
    # Expected outputs from BearDog logs
    expected_key = "2627605ded9551924defd62ee0ac7aa1"
    expected_iv = "e6221dda48a5626430510d78"
    
    print("Expected Outputs (from BearDog logs):")
    print("─" * 79)
    print(f"client_write_key (16 bytes): {expected_key}")
    print(f"client_write_iv (12 bytes):  {expected_iv}")
    print()
    
    # Derive key using HKDF-Expand-Label
    print("Derivation Process:")
    print("─" * 79)
    print("1. Deriving client_write_key:")
    print(f"   HKDF-Expand-Label(secret, 'key', '', 16)")
    
    derived_key = hkdf_expand_label(client_traffic_secret_0, "key", b"", 16)
    derived_key_hex = binascii.hexlify(derived_key).decode()
    
    print(f"   Result: {derived_key_hex}")
    print()
    
    print("2. Deriving client_write_iv:")
    print(f"   HKDF-Expand-Label(secret, 'iv', '', 12)")
    
    derived_iv = hkdf_expand_label(client_traffic_secret_0, "iv", b"", 12)
    derived_iv_hex = binascii.hexlify(derived_iv).decode()
    
    print(f"   Result: {derived_iv_hex}")
    print()
    
    # Validation
    print("═" * 79)
    print("📊 VALIDATION RESULTS")
    print("═" * 79)
    print()
    
    key_match = derived_key_hex == expected_key
    iv_match = derived_iv_hex == expected_iv
    
    print("client_write_key:")
    print(f"  Expected: {expected_key}")
    print(f"  Derived:  {derived_key_hex}")
    if key_match:
        print(f"  ✅ MATCH! HKDF-Expand-Label for 'key' is CORRECT!")
    else:
        print(f"  ❌ MISMATCH! HKDF-Expand-Label for 'key' has a BUG!")
    print()
    
    print("client_write_iv:")
    print(f"  Expected: {expected_iv}")
    print(f"  Derived:  {derived_iv_hex}")
    if iv_match:
        print(f"  ✅ MATCH! HKDF-Expand-Label for 'iv' is CORRECT!")
    else:
        print(f"  ❌ MISMATCH! HKDF-Expand-Label for 'iv' has a BUG!")
    print()
    
    # Final verdict
    print("═" * 79)
    print("🎯 FINAL VERDICT")
    print("═" * 79)
    print()
    
    if key_match and iv_match:
        print("✅ ✅ ✅ HKDF-Expand-Label is CORRECT! ✅ ✅ ✅")
        print()
        print("BearDog's key expansion implementation is RFC 8446 compliant!")
        print("The issue is NOT in HKDF-Expand-Label.")
        print()
        print("Next steps:")
        print("  • Validate transcript hash content")
        print("  • Validate Master Secret derivation")
        print("  • Check for subtle issues (byte order, sequence counter, etc.)")
    else:
        print("❌ ❌ ❌ HKDF-Expand-Label has a BUG! ❌ ❌ ❌")
        print()
        print("BearDog's key expansion is NOT RFC 8446 compliant!")
        print()
        print("Possible issues:")
        print("  • Label encoding ('tls13 ' prefix)")
        print("  • Length encoding (big-endian)")
        print("  • Context handling")
        print("  • HKDF-Expand implementation")
        print("  • Hash function (should be SHA-256 for cipher 0x1301)")
    print()
    print("═" * 79)

if __name__ == "__main__":
    main()


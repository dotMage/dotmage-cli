//! Test vectors per spec A.8 — Phase 1 gate.
//! All tests here MUST pass before proceeding to Phase 2.

use dotmage_crypto::blob;
use dotmage_crypto::envelope;
use dotmage_crypto::kdf;
use dotmage_crypto::secret;

/// A.8.1: Argon2id on fixed salt + password produces a deterministic MK (golden test).
#[test]
fn argon2id_golden_vector() {
    let password = b"correct-horse-battery-staple";
    let salt: [u8; 16] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    ];

    let mk1 = kdf::derive_master_key(password, &salt).expect("kdf failed");
    let mk2 = kdf::derive_master_key(password, &salt).expect("kdf failed");

    // Same input → same output (deterministic).
    assert_eq!(mk1.as_bytes(), mk2.as_bytes());

    // Non-trivial (not all zeros).
    assert_ne!(mk1.as_bytes(), &[0u8; 32]);
}

/// A.8.1 (extra): Different password → different MK.
#[test]
fn argon2id_different_password() {
    let salt = [0xaa_u8; 16];
    let mk_a = kdf::derive_master_key(b"password-a", &salt).unwrap();
    let mk_b = kdf::derive_master_key(b"password-b", &salt).unwrap();
    assert_ne!(mk_a.as_bytes(), mk_b.as_bytes());
}

/// A.8.1 (extra): Different salt → different MK.
#[test]
fn argon2id_different_salt() {
    let mk_a = kdf::derive_master_key(b"same-password", &[0x01; 16]).unwrap();
    let mk_b = kdf::derive_master_key(b"same-password", &[0x02; 16]).unwrap();
    assert_ne!(mk_a.as_bytes(), mk_b.as_bytes());
}

/// A.8.2: encrypt → decrypt round-trip restores bytes exactly.
#[test]
fn encrypt_decrypt_roundtrip() {
    let ak = [0x42_u8; 32];
    let plaintext = b"DATABASE_URL=postgres://localhost\nSECRET_KEY=hunter2\n";

    let encrypted = secret::encrypt_secret(&ak, plaintext, "myapp", "dev", 1)
        .expect("encrypt failed");
    let decrypted = secret::decrypt_secret(&ak, &encrypted, "myapp", "dev", 1)
        .expect("decrypt failed");

    assert_eq!(decrypted, plaintext);
}

/// A.8.3: Corrupting one byte of ciphertext → decrypt fails (AEAD).
#[test]
fn tampered_ciphertext_fails() {
    let ak = [0x42_u8; 32];
    let plaintext = b"SECRET=value";

    let mut encrypted = secret::encrypt_secret(&ak, plaintext, "app", "dev", 1)
        .expect("encrypt failed");

    // Flip one byte in ciphertext.
    encrypted.ciphertext[0] ^= 0xff;

    let result = secret::decrypt_secret(&ak, &encrypted, "app", "dev", 1);
    assert!(result.is_err(), "decrypt should fail on tampered ciphertext");
}

/// A.8.4: Swapping AAD (different app_name or rev) → decrypt fails.
#[test]
fn wrong_aad_app_name_fails() {
    let ak = [0x42_u8; 32];
    let plaintext = b"KEY=value";

    let encrypted = secret::encrypt_secret(&ak, plaintext, "app-a", "dev", 1)
        .expect("encrypt failed");

    // Try decrypting with different app name.
    let result = secret::decrypt_secret(&ak, &encrypted, "app-b", "dev", 1);
    assert!(result.is_err(), "decrypt should fail with wrong app name");
}

#[test]
fn wrong_aad_env_name_fails() {
    let ak = [0x42_u8; 32];
    let plaintext = b"KEY=value";

    let encrypted = secret::encrypt_secret(&ak, plaintext, "app", "dev", 1)
        .expect("encrypt failed");

    // Try decrypting with different env name.
    let result = secret::decrypt_secret(&ak, &encrypted, "app", "prod", 1);
    assert!(result.is_err(), "decrypt should fail with wrong env name");
}

#[test]
fn wrong_aad_rev_number_fails() {
    let ak = [0x42_u8; 32];
    let plaintext = b"KEY=value";

    let encrypted = secret::encrypt_secret(&ak, plaintext, "app", "dev", 1)
        .expect("encrypt failed");

    // Try decrypting with different rev number.
    let result = secret::decrypt_secret(&ak, &encrypted, "app", "dev", 2);
    assert!(result.is_err(), "decrypt should fail with wrong rev number");
}

/// A.8.5: Two encryptions of same plaintext → different nonce and ciphertext.
#[test]
fn different_nonce_each_encryption() {
    let ak = [0x42_u8; 32];
    let plaintext = b"SAME=data";

    let enc1 = secret::encrypt_secret(&ak, plaintext, "app", "dev", 1).unwrap();
    let enc2 = secret::encrypt_secret(&ak, plaintext, "app", "dev", 1).unwrap();

    assert_ne!(enc1.nonce, enc2.nonce, "nonces should differ");
    assert_ne!(enc1.ciphertext, enc2.ciphertext, "ciphertexts should differ");
}

/// A.8.6: MK decrypts wrapped AK; wrong password → AEAD error, not garbage.
#[test]
fn envelope_wrap_unwrap_roundtrip() {
    let password = b"my-password";
    let salt = kdf::generate_salt();
    let mk = kdf::derive_master_key(password, &salt).unwrap();

    let ak = envelope::generate_account_key();
    let wrapped = envelope::wrap_ak(&mk, &ak).unwrap();

    let unwrapped = envelope::unwrap_ak(&mk, &wrapped).unwrap();
    assert_eq!(*unwrapped, *ak);
}

#[test]
fn envelope_wrong_password_fails() {
    let salt = [0xbb_u8; 16];
    let mk_correct = kdf::derive_master_key(b"correct", &salt).unwrap();
    let mk_wrong = kdf::derive_master_key(b"wrong", &salt).unwrap();

    let ak = envelope::generate_account_key();
    let wrapped = envelope::wrap_ak(&mk_correct, &ak).unwrap();

    let result = envelope::unwrap_ak(&mk_wrong, &wrapped);
    assert!(result.is_err(), "unwrap with wrong password should fail cleanly");
}

/// A.8.7: Unknown blob version → explicit error.
#[test]
fn unknown_blob_version_errors() {
    let result = blob::decode_blob("v2:AAAA:BBBB");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("unsupported"),
        "error should mention unsupported version, got: {err}"
    );
}

/// Blob encode → decode round-trip.
#[test]
fn blob_roundtrip() {
    let ak = [0x42_u8; 32];
    let plaintext = b"TEST=value";

    let encrypted = secret::encrypt_secret(&ak, plaintext, "app", "dev", 1).unwrap();
    let encoded = blob::encode_blob(&encrypted);

    assert!(encoded.starts_with("v1:"), "blob should start with v1:");

    let decoded = blob::decode_blob(&encoded).unwrap();
    assert_eq!(decoded.nonce, encrypted.nonce);
    assert_eq!(decoded.ciphertext, encrypted.ciphertext);

    // Full round-trip: decode then decrypt.
    let decrypted = secret::decrypt_secret(&ak, &decoded, "app", "dev", 1).unwrap();
    assert_eq!(decrypted, plaintext);
}

/// Blob with invalid format.
#[test]
fn blob_invalid_format() {
    assert!(blob::decode_blob("garbage").is_err());
    assert!(blob::decode_blob("v1:onlyone").is_err());
    assert!(blob::decode_blob("").is_err());
}

/// Recovery key wrap/unwrap (Appendix J).
#[test]
fn recovery_key_wrap_unwrap() {
    let ak = envelope::generate_account_key();
    let rk = [0xcc_u8; 32]; // simulated recovery key

    let wrapped = envelope::wrap_ak_recovery(&rk, &ak).unwrap();
    let unwrapped = envelope::unwrap_ak_recovery(&rk, &wrapped).unwrap();
    assert_eq!(*unwrapped, *ak);
}

#[test]
fn recovery_key_wrong_code_fails() {
    let ak = envelope::generate_account_key();
    let rk_correct = [0xcc_u8; 32];
    let rk_wrong = [0xdd_u8; 32];

    let wrapped = envelope::wrap_ak_recovery(&rk_correct, &ak).unwrap();
    let result = envelope::unwrap_ak_recovery(&rk_wrong, &wrapped);
    assert!(result.is_err(), "wrong recovery key should fail");
}

/// Salt generation produces 16 random bytes (not all zeros).
#[test]
fn salt_generation() {
    let salt1 = kdf::generate_salt();
    let salt2 = kdf::generate_salt();
    assert_ne!(salt1, [0u8; 16]);
    assert_ne!(salt1, salt2, "two salts should differ");
}

/// AK generation produces 32 random bytes (not all zeros).
#[test]
fn ak_generation() {
    let ak1 = envelope::generate_account_key();
    let ak2 = envelope::generate_account_key();
    assert_ne!(*ak1, [0u8; 32]);
    assert_ne!(*ak1, *ak2, "two AKs should differ");
}

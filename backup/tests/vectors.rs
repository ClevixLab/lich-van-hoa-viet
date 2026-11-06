//! R6 (c): vector chuẩn cho hai thành phần mật mã, chạy qua đúng crate đã ghim.
//! - Argon2id: RFC 9106 §5.3 (m = 32 KiB, t = 3, p = 4, secret 8 × 0x03, AD 12 × 0x04, tag 32 byte).
//! - XChaCha20-Poly1305: draft-irtf-cfrg-xchacha-03 §A.3.1 (AEAD_XChaCha20_Poly1305).
//!
//! Cùng các giá trị được kiểm ở bản JavaScript (pwa/test/store/run.mjs).
use argon2::{Algorithm, Argon2, AssociatedData, ParamsBuilder, Version};
use chacha20poly1305::aead::{Aead, Payload};
use chacha20poly1305::{KeyInit, XChaCha20Poly1305};

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn argon2id_rfc9106_5_3() {
    let params = ParamsBuilder::new()
        .m_cost(32)
        .t_cost(3)
        .p_cost(4)
        .output_len(32)
        .data(AssociatedData::new(&[0x04; 12]).unwrap())
        .build()
        .unwrap();
    let a =
        Argon2::new_with_secret(&[0x03; 8], Algorithm::Argon2id, Version::V0x13, params).unwrap();
    let mut out = [0u8; 32];
    a.hash_password_into(&[0x01; 32], &[0x02; 16], &mut out)
        .unwrap();
    assert_eq!(out.to_vec(), hex("0d 64 0d f5 8d 78 76 6c 08 c0 37 a3 4a 8b 53 c9 d0 1e f0 45 2d 75 b6 5e b5 25 20 e9 6b 01 e6 59"));
}

const PT: &[u8] = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";

fn xchacha_key() -> Vec<u8> {
    (0x80u8..=0x9f).collect()
}
fn xchacha_nonce() -> Vec<u8> {
    (0x40u8..=0x57).collect()
}
const AAD: &str = "50515253c0c1c2c3c4c5c6c7";
const CT: &str = "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b4522f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff921f9664c97637da9768812f615c68b13b52e";
const TAG: &str = "c0875924c1c7987947deafd8780acf49";

#[test]
fn xchacha20poly1305_draft_a3() {
    let c = XChaCha20Poly1305::new_from_slice(&xchacha_key()).unwrap();
    let nonce = xchacha_nonce();
    let out = c
        .encrypt(
            nonce.as_slice().try_into().unwrap(),
            Payload {
                msg: PT,
                aad: &hex(AAD),
            },
        )
        .unwrap();
    let mut want = hex(CT);
    want.extend(hex(TAG));
    assert_eq!(out, want);
    let back = c
        .decrypt(
            nonce.as_slice().try_into().unwrap(),
            Payload {
                msg: &want,
                aad: &hex(AAD),
            },
        )
        .unwrap();
    assert_eq!(back, PT);
    // Lật một bit của thẻ, của bản mã, của AAD → từ chối.
    for i in [0usize, want.len() - 1] {
        let mut bad = want.clone();
        bad[i] ^= 1;
        assert!(c
            .decrypt(
                nonce.as_slice().try_into().unwrap(),
                Payload {
                    msg: &bad,
                    aad: &hex(AAD)
                }
            )
            .is_err());
    }
    let mut aad = hex(AAD);
    aad[0] ^= 1;
    assert!(c
        .decrypt(
            nonce.as_slice().try_into().unwrap(),
            Payload {
                msg: &want,
                aad: &aad
            }
        )
        .is_err());
}

/// Tham số của ứng dụng (03 §2.1) qua đúng hàm `derive_key` (khóa tham chiếu dùng chung với JS).
#[test]
fn derive_key_app_params_reference() {
    let k = lvhv_backup::derive_key("password", &[0u8; 16], 65536, 3, 4).unwrap();
    assert_eq!(
        k.to_vec(),
        hex("00b1eed9bee6dc0641a507717db76b6520ec876ece6cd10925e43875b543575e")
    );
}

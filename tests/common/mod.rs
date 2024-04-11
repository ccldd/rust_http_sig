use http_sig::{Signer, Verifier};
use indoc::indoc;
use p256::{
    ecdsa::{signature::Keypair, SigningKey, VerifyingKey},
    SecretKey,
};
use rstest::fixture;

const ECC_P256_TEST_KEY: &str = indoc! {"
    -----BEGIN EC PRIVATE KEY-----
    MHcCAQEEIFKbhfNZfpDsW43+0+JjUr9K+bTeuxopu653+hBaXGA7oAoGCCqGSM49
    AwEHoUQDQgAEqIVYZVLCrPZHGHjP17CTW0/+D9Lfw0EkjqF7xB4FivAxzic30tMM
    4GF+hR6Dxh71Z50VGGdldkkDXZCnTNnoXQ==
    -----END EC PRIVATE KEY-----"};

#[fixture]
pub fn ecc_p256_signing_key() -> SigningKey {
    let sk = SecretKey::from_sec1_pem(ECC_P256_TEST_KEY).unwrap();
    let key: SigningKey = SigningKey::from(sk);
    key
}

#[fixture]
pub fn ecc_p256_signer(ecc_p256_signing_key: SigningKey) -> Signer<SigningKey> {
    let signer = Signer::builder().with_key(ecc_p256_signing_key).build();
    signer
}

#[fixture]
pub fn ecc_p256_verifier(ecc_p256_signing_key: SigningKey) -> Verifier<VerifyingKey> {
    let verifier = Verifier::builder()
        .with_key(ecc_p256_signing_key.verifying_key().clone())
        .build();
    verifier
}

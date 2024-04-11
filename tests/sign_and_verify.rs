use common::{ecc_p256_signer, ecc_p256_verifier};
use http_sig::{SignRequest, Signer, Verifier, VerifyRequest};
use p256::ecdsa::{SigningKey, VerifyingKey};
use rstest::rstest;
mod common;

#[rstest]
fn test_sign_and_verify(
    ecc_p256_signer: Signer<SigningKey>,
    ecc_p256_verifier: Verifier<VerifyingKey>,
) {
    let mut request = http::Request::get("https://example.com").body(()).unwrap();
    ecc_p256_signer.sign(&mut request).unwrap();
    assert_ne!(request.headers().get("Signature").unwrap(), "");
    assert_ne!(request.headers().get("Signature-Input").unwrap(), "");

    ecc_p256_verifier.verify(&mut request).unwrap();
}

use base64::{engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD}, Engine as _};

pub fn decode(text: &str) -> Result<Vec<u8>, base64::DecodeError> {
    let mut decode = STANDARD.decode(text);
    if decode.is_err() {
        decode = STANDARD_NO_PAD.decode(text);
    }
    if decode.is_err() {
        decode = URL_SAFE.decode(text);
    }
    if decode.is_err() {
        decode = URL_SAFE_NO_PAD.decode(text);
    }
    decode
}
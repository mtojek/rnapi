use encoding_rs::WINDOWS_1250;

pub fn to_utf8(data: &[u8]) -> Vec<u8> {
    let (decoded, _, _) = WINDOWS_1250.decode(data);
    decoded.as_bytes().into()
}

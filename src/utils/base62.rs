use aes::Aes256;
use base62;
use fpe::{
    self,
    ff1::{BinaryNumeralString, FF1},
};

pub fn encode(database_id: u64) -> String {
    let obfuscated_id = obfuscate(database_id);
    let encrypted = base62::encode(obfuscated_id);
    encrypted
}

fn obfuscate(pt: u64) -> u64 {
    let bytes = pt.to_le_bytes();
    let numeral = BinaryNumeralString::from_bytes_le(&bytes);
    let key = [0; 32];
    let radix = 2;
    let ff = FF1::<Aes256>::new(&key, radix).unwrap();
    let ct = ff.encrypt(&[], &numeral).unwrap();
    let ct_bytes = ct.to_bytes_le();
    let encrypted_number = u64::from_le_bytes(ct_bytes.try_into().unwrap());
    encrypted_number
}

pub fn decode(short_code: String) -> u64{
    let key = [0; 32];
    let radix = 2;
    let ff = FF1::<Aes256>::new(&key, radix).unwrap();
    let encoded_id = base62::decode(short_code).unwrap() as u64;
    let encrypted_bytes = encoded_id.to_le_bytes();
    let encrypted_numeral = BinaryNumeralString::from_bytes_le(&encrypted_bytes);
    let decrypted = ff.decrypt(&[], &encrypted_numeral).unwrap();
    let decrypted_bytes = decrypted.to_bytes_le();
    let decrypted_number = u64::from_le_bytes(decrypted_bytes.try_into().unwrap());
    decrypted_number
}

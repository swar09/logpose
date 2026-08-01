use aes::Aes256;
use base62;
use fpe::{
    self,
    ff1::{FF1, FlexibleNumeralString},
};
const RADIX: u32 = 3812;
const LENGTH: usize = 2;
const RL: u32 = RADIX.pow(LENGTH as u32);

const LOWER_BOUND: u32 = 62_u32.pow(3);
const UPPER_BOUND: u32 = 62_u32.pow(4) - 1;
const RANGE_SIZE: u32 = UPPER_BOUND - LOWER_BOUND + 1;

pub fn encode(database_id: u32, ff: &FF1<Aes256>) -> String {
    assert!(database_id < RL, "DATABASE_ID out of RL");
    let obfuscated_shifted_id = obfuscate(database_id, ff);
    base62::encode(obfuscated_shifted_id)
}

pub fn obfuscate(database_id: u32, ff: &FF1<Aes256>) -> u32 {
    let digits = integer_to_digits(database_id, LENGTH);
    let numeral_string = FlexibleNumeralString::from(digits);
    let ct = ff.encrypt(&[], &numeral_string).unwrap();
    let encrypted_number = digits_to_integer(ct.into());
    encrypted_number + LOWER_BOUND
}

pub fn deobfuscate(encrypted_shifted_number: u32, ff: &FF1<Aes256>) -> u32 {
    let digits = integer_to_digits(encrypted_shifted_number - LOWER_BOUND, LENGTH);
    let numeral_string = FlexibleNumeralString::from(digits);
    let pt = ff.decrypt(&[], &numeral_string).unwrap();
    let decrypted_number = digits_to_integer(pt.into());
    decrypted_number
}

pub fn decode(short_code: String, ff: &FF1<Aes256>) -> u32 {
    let encrypted_number = base62::decode(short_code).unwrap() as u32;
    deobfuscate(encrypted_number, ff)
}

fn integer_to_digits(mut val: u32, length_var: usize) -> Vec<u16> {
    let mut digits: Vec<u16> = vec![0; length_var];
    for i in (0..length_var).rev() {
        digits[i] = (val % RADIX) as u16;
        val /= RADIX;
    }
    digits
}

fn digits_to_integer(digits: Vec<u16>) -> u32 {
    let mut val: u32 = 0;
    for digit in digits {
        val = val * RADIX + digit as u32;
    }
    val
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn obfuscate_inverse() {
        let dummy_key_string = String::from("abcdefghijklmnopqrstuvwxyz123456");
        let key = dummy_key_string.as_bytes();
        let ff = FF1::<Aes256>::new(key, RADIX).unwrap();
        // assert!(deobfuscate(obfuscate(1234567 as u32, key), key), 1234567);
        assert_eq!(deobfuscate(obfuscate(12345678, &ff), &ff), 12345678);
    }
}

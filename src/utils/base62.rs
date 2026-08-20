use aes::Aes256;
use base62;
use fpe::ff1::{FF1, FlexibleNumeralString};

const RADIX: u32 = 3812;
const LENGTH: usize = 2;
const RL: u32 = RADIX.pow(LENGTH as u32);

const LOWER_BOUND: u32 = 62_u32.pow(3);
#[allow(dead_code)]
const UPPER_BOUND: u32 = 62_u32.pow(4) - 1;
#[allow(dead_code)]
const RANGE_SIZE: u32 = UPPER_BOUND - LOWER_BOUND + 1;

pub fn encode(database_id: u32, ff: &FF1<Aes256>) -> Result<String, crate::error::AppError> {
    if database_id >= RL {
        return Err(crate::error::AppError::Internal(format!(
            "Database ID {} exceeds maximum limit of {} for 4-character short codes",
            database_id, RL
        )));
    }
    let obfuscated_shifted_id = obfuscate(database_id, ff);
    Ok(base62::encode(obfuscated_shifted_id))
}

pub fn decode(short_code: &str, ff: &FF1<Aes256>) -> Result<u32, crate::error::AppError> {
    let encrypted_number = base62::decode(short_code)
        .map_err(|_| crate::error::AppError::BadRequest("Invalid short code".to_string()))?
        as u32;
    deobfuscate(encrypted_number, ff)
}

pub fn obfuscate(database_id: u32, ff: &FF1<Aes256>) -> u32 {
    let digits = integer_to_digits(database_id, LENGTH);
    let numeral_string = FlexibleNumeralString::from(digits);
    let ct = ff.encrypt(&[], &numeral_string).unwrap();
    let encrypted_number = digits_to_integer(ct.into());
    encrypted_number + LOWER_BOUND
}

pub fn deobfuscate(encrypted_shifted_number: u32, ff: &FF1<Aes256>) -> Result<u32, crate::error::AppError> {
    if encrypted_shifted_number < LOWER_BOUND {
        return Err(crate::error::AppError::BadRequest("Invalid short code".to_string()));
    }
    let digits = integer_to_digits(encrypted_shifted_number - LOWER_BOUND, LENGTH);
    let numeral_string = FlexibleNumeralString::from(digits);
    let pt = ff
        .decrypt(&[], &numeral_string)
        .map_err(|_| crate::error::AppError::BadRequest("Failed to decrypt short code".to_string()))?;

    Ok(digits_to_integer(pt.into()))
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

    fn get_test_ff1() -> FF1<Aes256> {
        let key = b"abcdefghijklmnopqrstuvwxyz123456";
        FF1::<Aes256>::new(key, RADIX).unwrap()
    }

    #[test]
    fn test_obfuscate_inverse() {
        let ff = get_test_ff1();
        let test_id = 123456;
        let obfuscated = obfuscate(test_id, &ff);
        let deobfuscated = deobfuscate(obfuscated, &ff).unwrap();
        assert_eq!(deobfuscated, test_id);
    }

    #[test]
    fn test_encode_and_decode_roundtrip() {
        let ff = get_test_ff1();
        let test_ids = vec![0, 1, 42, 9999, 1400000];

        for &id in &test_ids {
            let code = encode(id, &ff).expect("Failed to encode");
            let decoded_id = decode(&code, &ff).expect("Failed to decode valid short code");
            assert_eq!(decoded_id, id);
        }
    }

    #[test]
    fn test_decode_invalid_characters() {
        let ff = get_test_ff1();
        let invalid_code = "!!!invalid_base62!!!";
        assert!(decode(invalid_code, &ff).is_err());
    }

    #[test]
    fn test_deobfuscate_below_lower_bound() {
        let ff = get_test_ff1();
        let value_below_lower_bound = LOWER_BOUND - 1;
        assert!(deobfuscate(value_below_lower_bound, &ff).is_err());
    }
}

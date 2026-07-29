use aes::Aes256;
use base62;
use fpe::{
    self,
    ff1::{FF1, FlexibleNumeralString},
};

const RADIX: u32 = 3813;
const LENGTH: usize = 2;

const LOWER_BOUND: u32 = 238_328;
const UPPER_BOUND: u32 = 14_776_335;
const RANGE_SIZE: u32 = UPPER_BOUND - LOWER_BOUND + 1; // 14,538,008

pub fn encode(database_id: u32) -> String {
    if database_id > RANGE_SIZE {
        assert!(database_id < RANGE_SIZE , "DATABASE_ID out of RANGE");
    }
    let obfuscated_id = obfuscate(database_id);
    let encrypted = base62::encode(obfuscated_id);
    encrypted
}

pub fn obfuscate(mut number: u32) -> u32 {
    let key = [0; 32]; // use dotenvy for secure key 

    loop {
        let digits = integer_to_digits(number, LENGTH);
        let ff = FF1::<Aes256>::new(&key, RADIX).unwrap();

        let numeral_string = FlexibleNumeralString::from(digits);
        let ct = ff.encrypt(&[], &numeral_string).unwrap();
        let encrypted_number = digits_to_integer(ct.into());

        if encrypted_number < UPPER_BOUND && encrypted_number > LOWER_BOUND {
            return encrypted_number;
        }
        number = encrypted_number;
    }
}

pub fn deobfuscate(mut encrypted_number: u32) -> u32 {
    let key = [0; 32]; // use dotenvy for secure key 

    loop {
        let digits = integer_to_digits(encrypted_number, LENGTH);
        let ff = FF1::<Aes256>::new(&key, RADIX).unwrap();
    
        let numeral_string = FlexibleNumeralString::from(digits);
        let pt = ff.decrypt(&[], &numeral_string).unwrap();
        let decrypted_number = digits_to_integer(pt.into());
        
        
         if decrypted_number < RANGE_SIZE && decrypted_number > 0 {
            return decrypted_number;
        }
        encrypted_number = decrypted_number;
    }
}

pub fn decode(short_code: String) -> u32 {
    let encrypted_number = base62::decode(short_code).unwrap() as u32;
    let database_id = deobfuscate(encrypted_number);
    database_id
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

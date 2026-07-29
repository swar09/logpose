use md5::{Digest, Md5};
pub fn genrate(long_url: String, user_id: String) -> String {
    let data = long_url + &user_id;
    let byte_array = Md5::digest(data);
    let hash = hex::encode(byte_array);
    hash
}



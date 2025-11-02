use itertools::Itertools;
use sha2::{Digest, Sha256};

/// Allows hashing the object to a string containing only chars a-z
pub trait AZHash {
    /// hashes the object to a string with chars a-z
    fn az_hash(&self) -> String;
}

const U64_MAX_AZ_HASH_LEN: usize = 14;

impl<T: AsRef<[u8]>> AZHash for T {
    /// sha-256 hashes and output the hash as text with chars a-z
    fn az_hash(&self) -> String {
        let hash = Sha256::digest(self.as_ref());

        let mut result = String::new();
        for group in &hash.iter().chunks(8) {
            let mut bytes = [0u8; 8];
            let mut count: u8 = 0;
            for (place, value) in bytes.iter_mut().zip(group) {
                *place = *value;
                count += 1;
            }
            while count < 8 {
                bytes[count as usize] = 0;
                count += 1;
            }

            let mut number = u64::from_le_bytes(bytes);

            let mut az_decoded = [0u8; U64_MAX_AZ_HASH_LEN];
            for element in az_decoded.iter_mut() {
                *element = (number % 26) as u8;
                number /= 26;
            }
            assert_eq!(
                number, 0,
                "The number is now zero. log(U64::MAX)/log(26)=14, see test below"
            );

            for element in az_decoded.iter() {
                result.push((b'a' + *element) as char);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn max_test() {
        assert!(((u64::MAX as u128) + 1).ilog(26) < U64_MAX_AZ_HASH_LEN as u32);
    }
}

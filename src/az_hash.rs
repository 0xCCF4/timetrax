use digest::Digest;
use itertools::Itertools;

/// Allows hashing the object to a string containing only chars a-z
pub trait AZHash {
    /// hashes the object to a string with chars a-z
    fn az_hash<D: Digest>(&self) -> String;

    /// SHA256 hashes the object to a string with chars a-z
    fn az_hash_sha256(&self) -> String {
        self.az_hash::<sha2::Sha256>()
    }

    /// SHA512 hashes the object to a string with chars a-z
    fn az_hash_sha512(&self) -> String {
        self.az_hash::<sha2::Sha512>()
    }
}

const U64_MAX_AZ_HASH_LEN: usize = 14;

impl<T: AsRef<[u8]>> AZHash for T {
    /// hashes and output the hash as text with chars a-z
    fn az_hash<D: Digest>(&self) -> String {
        let mut digest = D::new();
        digest.update(self.as_ref());
        let output = digest.finalize();
        let hash: &[u8] = output.as_ref();

        let mut result = String::new();
        for group in &hash.iter().chunks(8) {
            // process 8x8 bits = 64 bits at once
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

use argon2::{
    Argon2, Params,
    password_hash::{
        Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};
use std::sync::OnceLock;

pub struct PasswordManager;

static INSTANCE: OnceLock<Argon2> = OnceLock::new();

impl PasswordManager {
    fn engine() -> &'static Argon2<'static> {
        INSTANCE.get_or_init(|| {
            let params = Params::new(
                64 * 1024, // 64MB Memory (m)
                3,         // 3 Iterations (t)
                4,         // 4 Parallelism lanes (p)
                None,      // Default hash length (32 bytes)
            )
            .expect("Invalid Argon2 parameters");

            Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
        })
    }

    pub fn hash_password(password: &str) -> Result<String, Error> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Self::engine().hash_password(password.as_bytes(), &salt)?;

        Ok(hash.to_string())
    }

    pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, Error> {
        let parsed_hash = PasswordHash::new(stored_hash)?;

        let result = Self::engine().verify_password(password.as_bytes(), &parsed_hash);

        match result {
            Ok(_) => Ok(true),
            Err(Error::Password) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

//! Argon2id 密码哈希 — 用于 public 端手机号+密码登录。

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use hone_core::{HoneError, HoneResult};
use password_hash::{SaltString, rand_core::OsRng};

pub fn hash_password(plain: &str) -> HoneResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| HoneError::Storage(format!("argon2 hash 失败: {e}")))
}

pub fn verify_password(plain: &str, encoded: &str) -> HoneResult<bool> {
    let parsed = match PasswordHash::new(encoded) {
        Ok(p) => p,
        Err(e) => return Err(HoneError::Storage(format!("argon2 解析失败: {e}"))),
    };
    match Argon2::default().verify_password(plain.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(password_hash::Error::Password) => Ok(false),
        Err(e) => Err(HoneError::Storage(format!("argon2 验证失败: {e}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_roundtrip() {
        let hash = hash_password("hunter2-password").expect("hash ok");
        assert!(verify_password("hunter2-password", &hash).expect("verify ok"));
    }

    #[test]
    fn wrong_password_returns_false() {
        let hash = hash_password("correct-horse-battery").expect("hash ok");
        assert!(!verify_password("staple", &hash).expect("verify ok"));
    }

    #[test]
    fn salt_is_randomized_per_hash() {
        let a = hash_password("same-input").expect("hash ok");
        let b = hash_password("same-input").expect("hash ok");
        assert_ne!(a, b, "salt should randomize encoded hash");
        assert!(verify_password("same-input", &a).unwrap());
        assert!(verify_password("same-input", &b).unwrap());
    }

    #[test]
    fn malformed_hash_returns_err() {
        let res = verify_password("x", "not-a-valid-hash");
        assert!(res.is_err());
    }
}

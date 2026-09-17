//! Shamir's Secret Sharing for key recovery.
//!
//! The master secret is split into N shares with threshold T.
//! Any T shares can reconstruct the secret; fewer than T cannot.

use rand::RngCore;
use serde::{Deserialize, Serialize};
use crate::error::RecoveryError;

#[derive(Debug, Serialize, Deserialize)]
pub struct SssConfig {
    pub threshold: u32,
    pub total_shares: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SssShare {
    pub share_id: u32,
    pub data: Vec<u8>,
}

/// Splits a secret into N shares with threshold T.
pub fn split(secret: &[u8], config: &SssConfig) -> Result<Vec<SssShare>, RecoveryError> {
    if config.threshold < 2 {
        return Err(RecoveryError::InvalidShardData);
    }
    if config.threshold > config.total_shares {
        return Err(RecoveryError::InvalidShardData);
    }

    let n = config.total_shares as usize;
    let t = config.threshold as usize;
    let mut shares: Vec<Vec<u8>> = vec![vec![]; n];

    for &byte in secret {
        let mut coeffs: Vec<u8> = vec![byte];
        for _ in 1..t {
            coeffs.push(rand::random());
        }
        for (i, share) in shares.iter_mut().enumerate() {
            let x = (i + 1) as u8;
            share.push(evaluate(&coeffs, x));
        }
    }

    Ok(shares
        .into_iter()
        .enumerate()
        .map(|(i, data)| SssShare { share_id: (i + 1) as u32, data })
        .collect())
}

/// Reconstructs the secret from T or more shares.
pub fn reconstruct(shares: &[SssShare], threshold: u32) -> Result<Vec<u8>, RecoveryError> {
    if shares.len() < threshold as usize {
        return Err(RecoveryError::InsufficientShares {
            have: shares.len() as u32,
            need: threshold,
        });
    }

    // Check for duplicates
    let mut seen = std::collections::HashSet::new();
    for s in shares {
        if !seen.insert(s.share_id) {
            return Err(RecoveryError::DuplicateShare(s.share_id));
        }
    }

    let first_share_len = shares[0].data.len();
    let mut secret = vec![0u8; first_share_len];

    for (byte_idx, _) in secret.iter_mut().enumerate() {
        let points: Vec<(u8, u8)> = shares
            .iter()
            .map(|s| (s.share_id as u8, s.data[byte_idx]))
            .collect();

        secret[byte_idx] = lagrange_interpolate_at_zero(&points);
    }

    Ok(secret)
}

fn evaluate(coeffs: &[u8], x: u8) -> u8 {
    let mut result: u8 = 0;
    for &c in coeffs.iter().rev() {
        result = gf256_mul(result, x) ^ c;
    }
    result
}

fn gf256_mul(a: u8, b: u8) -> u8 {
    let mut p: u16 = 0;
    let mut a = a as u16;
    let mut b = b as u16;
    for _ in 0..8 {
        if b & 1 != 0 { p ^= a; }
        let hi = a & 0x80;
        a <<= 1;
        if hi != 0 { a ^= 0x11b; }
        b >>= 1;
    }
    p as u8
}

fn gf256_inv(a: u8) -> u8 {
    if a == 0 { return 0; }
    let mut result = a;
    let mut power = 254u8; // a^254 = a^-1 in GF(256)
    while power != 0 {
        if power & 1 != 0 { result = gf256_mul(result, result); }
        power >>= 1;
    }
    result
}

fn lagrange_interpolate_at_zero(points: &[(u8, u8)]) -> u8 {
    let mut result: u8 = 0;
    for (i, &(xi, yi)) in points.iter().enumerate() {
        let mut numerator: u8 = 1;
        let mut denominator: u8 = 1;
        for (j, &(xj, _)) in points.iter().enumerate() {
            if i == j { continue; }
            // L_i(0) = prod_{j!=i} (0 - x_j) / (x_i - x_j)
            // In GF(256): subtraction = XOR
            numerator = gf256_mul(numerator, xj); // 0 ^ xj = xj
            denominator = gf256_mul(denominator, xi ^ xj);
        }
        let lagrange_coeff = gf256_mul(numerator, gf256_inv(denominator));
        result ^= gf256_mul(yi, lagrange_coeff);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_reconstruct() {
        let secret = b"my_super_secret_key_32_bytes!!";
        let config = SssConfig { threshold: 2, total_shares: 3 };
        let shares = split(secret, &config).unwrap();
        assert_eq!(shares.len(), 3);

        // Reconstruct with any 2 shares
        let recovered = reconstruct(&[shares[0].clone(), shares[1].clone()], 2).unwrap();
        assert_eq!(recovered, secret);

        let recovered = reconstruct(&[shares[0].clone(), shares[2].clone()], 2).unwrap();
        assert_eq!(recovered, secret);

        let recovered = reconstruct(&[shares[1].clone(), shares[2].clone()], 2).unwrap();
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_insufficient_shares() {
        let secret = b"secret";
        let config = SssConfig { threshold: 3, total_shares: 5 };
        let shares = split(secret, &config).unwrap();

        let result = reconstruct(&shares[..2], 3);
        assert!(matches!(result, Err(RecoveryError::InsufficientShares { .. })));
    }
}   
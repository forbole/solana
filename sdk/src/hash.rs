pub use solana_program::hash::*;

/// random hash value for tests and benchmarks.
#[cfg(any(feature = "full", target_arch = "wasm32"))]
pub fn new_rand<R: ?Sized>(rng: &mut R) -> Hash
where
    R: rand::Rng,
{
    let mut buf = [0u8; HASH_BYTES];
    rng.fill(&mut buf);
    Hash::new(&buf)
}

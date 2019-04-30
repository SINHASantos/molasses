use crate::{
    crypto::{
        aead::{AuthenticatedEncryption, AES128GCM_IMPL},
        dh::{DhPrivateKey, DhPublicKey, DiffieHellman, P256_IMPL, X25519_IMPL},
    },
    error::Error,
};

/// This represents the X25519-SHA256-AES128GCM ciphersuite
pub const X25519_SHA256_AES128GCM: CipherSuite = CipherSuite {
    name: "X25519_SHA256_AES128GCM",
    dh_impl: &X25519_IMPL,
    aead_impl: &AES128GCM_IMPL,
    hash_alg: &ring::digest::SHA256,
};

pub(crate) const P256_SHA256_AES128GCM: CipherSuite = CipherSuite {
    name: "P256_SHA256_AES128GCM",
    dh_impl: &P256_IMPL,
    aead_impl: &AES128GCM_IMPL,
    hash_alg: &ring::digest::SHA256,
};

/// Represents the contents of an MLS ciphersuite: a DH-like key-agreement protocol, a
/// hashing algorithm, and an authenticated encryption algorithm.
pub struct CipherSuite {
    /// The name of this cipher suite
    pub(crate) name: &'static str,

    /// The trait object that implements our key exchange functionality
    pub(crate) dh_impl: &'static dyn DiffieHellman,

    /// The trait object that implements our authenticated encryption functionality
    pub(crate) aead_impl: &'static dyn AuthenticatedEncryption,

    /// The `ring::digest::Algorithm` that implements our hashing functionality
    // We're gonna have to break the mold here. Originally this was Hash: digest::Digest. But to
    // define HKDF and HMAC over a generic Digest, one needs the following constraints:
    //     Hash: Input + BlockInput + FixedOutput + Reset + Default + Clone,
    //     Hash::BlockSize: ArrayLength<u8> + Clone,
    //     Hash::OutputSize: ArrayLength<u8>
    // and I'm not about to do that. Idea for the future: come back to using something like Hash,
    // but we can kill off all the ArrayLength stuff once associated constants for array lengths
    // becomes possible. Until then, we're probably just gonna use Vecs. The other downside is that
    // using a const locks us into whatever ring implements. Currently, it's just the SHA2 family.
    pub(crate) hash_alg: &'static ring::digest::Algorithm,
}

// TODO: Remove this impl if Add messages come with public_key indices in the future
// CipherSuites are uniquely identified by their tags. We need this in order to dedup ciphersuite
// lists in UserInitKeys
impl PartialEq for CipherSuite {
    fn eq(&self, other: &CipherSuite) -> bool {
        self.name.eq(other.name)
    }
}

impl CipherSuite {
    /// Given an arbitrary number of bytes, derives a Diffie-Hellman keypair. For this ciphersuite,
    /// the function is simply `scalar: [0u8; 32] = SHA256(bytes)`.
    pub(crate) fn derive_key_pair(
        &self,
        bytes: &[u8],
    ) -> Result<(DhPublicKey, DhPrivateKey), Error> {
        let digest = ring::digest::digest(self.hash_alg, bytes);
        let scalar_bytes = digest.as_ref();

        let privkey = self.dh_impl.private_key_from_bytes(scalar_bytes)?;
        let pubkey = self.dh_impl.derive_public_key(&privkey);

        Ok((pubkey, privkey))
    }
}

impl core::fmt::Debug for CipherSuite {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // Ensure that the secret value isn't accidentally logged
        f.write_str(self.name)
    }
}

// Miniscript
// Written in 2019 by
//     Sanket Kanjular and Andrew Poelstra
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

use dogecoin::hashes::{hash160, hex::ToHex};
use dogecoin::{self, secp256k1};
use std::{error, fmt};

/// Detailed Error type for Interpreter
#[derive(Debug)]
pub enum Error {
    /// Could not satisfy, absolute locktime not met
    AbsoluteLocktimeNotMet(u32),
    /// General Interpreter error.
    CouldNotEvaluate,
    /// We expected a push (including a `OP_1` but no other numeric pushes)
    ExpectedPush,
    /// The preimage to the hash function must be exactly 32 bytes.
    HashPreimageLengthMismatch,
    /// Incorrect scriptPubKey (pay-to-pubkeyhash) for the provided public key
    IncorrectPubkeyHash,
    /// Incorrect scriptPubKey for the provided redeem script
    IncorrectScriptHash,
    /// Incorrect scriptPubKey (pay-to-witness-pubkeyhash) for the provided public key
    IncorrectWPubkeyHash,
    /// Incorrect scriptPubKey for the provided witness script
    IncorrectWScriptHash,
    /// MultiSig missing at least `1` witness elements out of `k + 1` required
    InsufficientSignaturesMultiSig,
    /// Signature failed to verify
    InvalidSignature(dogecoin::PublicKey),
    /// Last byte of this signature isn't a standard sighash type
    NonStandardSigHash(Vec<u8>),
    /// Miniscript error
    Miniscript(::Error),
    /// MultiSig requires 1 extra zero element apart from the `k` signatures
    MissingExtraZeroMultiSig,
    /// Script abortion because of incorrect dissatisfaction for multisig.
    /// Any input witness apart from sat(0 sig ...) or nsat(0 0 ..) leads to
    /// this error. This is network standardness assumption and miniscript only
    /// supports standard scripts
    MultiSigEvaluationError,
    ///Witness must be empty for pre-segwit transactions
    NonEmptyWitness,
    ///ScriptSig must be empty for pure segwit transactions
    NonEmptyScriptSig,
    /// Script abortion because of incorrect dissatisfaction for Checksig.
    /// Any input witness apart from sat(sig) or nsat(0) leads to
    /// this error. This is network standardness assumption and miniscript only
    /// supports standard scripts
    PkEvaluationError(dogecoin::PublicKey),
    /// The Public Key hash check for the given pubkey. This occurs in `PkH`
    /// node when the given key does not match to Hash in script.
    PkHashVerifyFail(hash160::Hash),
    /// Parse Error while parsing a `stack::Element::Push` as a Pubkey. Both
    /// 33 byte and 65 bytes are supported.
    PubkeyParseError,
    /// Could not satisfy, relative locktime not met
    RelativeLocktimeNotMet(u32),
    /// Forward-secp related errors
    Secp(secp256k1::Error),
    /// Miniscript requires the entire top level script to be satisfied.
    ScriptSatisfactionError,
    /// An uncompressed public key was encountered in a context where it is
    /// disallowed (e.g. in a Segwit script or p2wpkh output)
    UncompressedPubkey,
    /// Got `stack::Element::Satisfied` or `stack::Element::Dissatisfied` when the
    /// interpreter was expecting `stack::Element::Push`
    UnexpectedStackBoolean,
    /// Unexpected Stack End, caused by popping extra elements from stack
    UnexpectedStackEnd,
    /// Unexpected Stack Push `stack::Element::Push` element when the interpreter
    /// was expecting a stack boolean `stack::Element::Satisfied` or
    /// `stack::Element::Dissatisfied`
    UnexpectedStackElementPush,
    /// Verify expects stack top element exactly to be `stack::Element::Satisfied`.
    /// This error is raised even if the stack top is `stack::Element::Push`.
    VerifyFailed,
}

#[doc(hidden)]
impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Error {
        Error::Secp(e)
    }
}

#[doc(hidden)]
impl From<::Error> for Error {
    fn from(e: ::Error) -> Error {
        Error::Miniscript(e)
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Secp(ref err) => Some(err),
            ref x => Some(x),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::AbsoluteLocktimeNotMet(n) => write!(
                f,
                "required absolute locktime CLTV of {} blocks, not met",
                n
            ),
            Error::ExpectedPush => f.write_str("expected push in script"),
            Error::CouldNotEvaluate => f.write_str("Interpreter Error: Could not evaluate"),
            Error::HashPreimageLengthMismatch => f.write_str("Hash preimage should be 32 bytes"),
            Error::IncorrectPubkeyHash => f.write_str("public key did not match scriptpubkey"),
            Error::IncorrectScriptHash => f.write_str("redeem script did not match scriptpubkey"),
            Error::IncorrectWPubkeyHash => {
                f.write_str("public key did not match scriptpubkey (segwit v0)")
            }
            Error::IncorrectWScriptHash => f.write_str("witness script did not match scriptpubkey"),
            Error::InsufficientSignaturesMultiSig => f.write_str("Insufficient signatures for CMS"),
            Error::InvalidSignature(pk) => write!(f, "bad signature with pk {}", pk),
            Error::NonStandardSigHash(ref sig) => {
                write!(
                    f,
                    "Non standard sighash type for signature '{}'",
                    sig.to_hex()
                )
            }
            Error::NonEmptyWitness => f.write_str("legacy spend had nonempty witness"),
            Error::NonEmptyScriptSig => f.write_str("segwit spend had nonempty scriptsig"),
            Error::Miniscript(ref e) => write!(f, "parse error: {}", e),
            Error::MissingExtraZeroMultiSig => f.write_str("CMS missing extra zero"),
            Error::MultiSigEvaluationError => {
                f.write_str("CMS script aborted, incorrect satisfaction/dissatisfaction")
            }
            Error::PkEvaluationError(ref key) => write!(f, "Incorrect Signature for pk {}", key),
            Error::PkHashVerifyFail(ref hash) => write!(f, "Pubkey Hash check failed {}", hash),
            Error::PubkeyParseError => f.write_str("could not parse pubkey"),
            Error::RelativeLocktimeNotMet(n) => {
                write!(f, "required relative locktime CSV of {} blocks, not met", n)
            }
            Error::ScriptSatisfactionError => f.write_str("Top level script must be satisfied"),
            Error::Secp(ref e) => fmt::Display::fmt(e, f),
            Error::UncompressedPubkey => {
                f.write_str("uncompressed pubkey in non-legacy descriptor")
            }
            Error::UnexpectedStackBoolean => {
                f.write_str("Expected Stack Push operation, found stack bool")
            }
            Error::UnexpectedStackElementPush => write!(f, "Got {}, expected Stack Boolean", 1),
            Error::UnexpectedStackEnd => f.write_str("unexpected end of stack"),
            Error::VerifyFailed => {
                f.write_str("Expected Satisfied Boolean at stack top for VERIFY")
            }
        }
    }
}

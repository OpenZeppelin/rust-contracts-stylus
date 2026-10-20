//! Reverting the state changes of a failed call.
use core::fmt::Debug;

use crate::Env;

/// Revert helpers for the `Result` of a contract call.
///
/// A call made through [`Contract::sender`](crate::Contract::sender) keeps
/// its state changes even when it returns `Err`, like a Rust method does.
/// Use these methods where a transaction would revert: they restore the
/// storage, balances and logs recorded before the call and hand back the
/// error.
pub trait ResultExt<T, E> {
    /// Reverts the last call and returns its error.
    ///
    /// # Panics
    ///
    /// If the call succeeded.
    fn unwrap_revert(self) -> E;

    /// Reverts the last call and returns its error.
    ///
    /// # Panics
    ///
    /// If the call succeeded, with `msg` in the panic message.
    fn expect_revert(self, msg: &str) -> E;
}

impl<T, E: Debug> ResultExt<T, E> for Result<T, E> {
    #[track_caller]
    fn unwrap_revert(self) -> E {
        match self {
            Ok(_) => panic!("{}", Env::current().unexpected_success()),
            Err(err) => {
                Env::current().revert_pending();
                err
            }
        }
    }

    #[track_caller]
    fn expect_revert(self, msg: &str) -> E {
        match self {
            Ok(_) => panic!("{msg}: {}", Env::current().unexpected_success()),
            Err(err) => {
                Env::current().revert_pending();
                err
            }
        }
    }
}

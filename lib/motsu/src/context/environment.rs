//! Module with unit test EVM environment for Stylus contracts.

/// Block Timestamp - Epoch timestamp: 1st January 2025 `00::00::00`.
const BLOCK_TIMESTAMP: u64 = 1_735_689_600;
/// Arbitrum's CHAID ID.
const CHAIN_ID: u64 = 42161;

/// Dummy contract address set for tests.
const CONTRACT_ADDRESS: &[u8; 42] =
    b"0xdCE82b5f92C98F27F116F70491a487EFFDb6a2a9";

/// Externally Owned Account (EOA) code hash.
const EOA_CODEHASH: &[u8; 66] =
    b"0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";

/// Dummy msg sender set for tests.
const MSG_SENDER: &[u8; 42] = b"0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF";

pub(crate) struct Environment {
    account_codehash: [u8; 66],
    block_timestamp: u64,
    chain_id: u64,
    contract_address: [u8; 42],
    events: Vec<Vec<u8>>,
    msg_sender: [u8; 42],
}

impl Default for Environment {
    /// Creates default environment for a test case.
    fn default() -> Environment {
        Self {
            account_codehash: *EOA_CODEHASH,
            block_timestamp: BLOCK_TIMESTAMP,
            chain_id: CHAIN_ID,
            contract_address: *CONTRACT_ADDRESS,
            events: Vec::new(),
            msg_sender: *MSG_SENDER,
        }
    }
}

impl Environment {
    /// Gets the code hash of the account at the given address.
    pub(crate) fn account_codehash(&self) -> [u8; 66] {
        self.account_codehash
    }

    /// Gets a bounded estimate of the Unix timestamp at which the Sequencer
    /// sequenced the transaction.
    pub(crate) fn block_timestamp(&self) -> u64 {
        self.block_timestamp
    }

    /// Gets the chain ID of the current chain.
    pub(crate) fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Gets the address of the current program.
    pub(crate) fn contract_address(&self) -> [u8; 42] {
        self.contract_address
    }

    /// Gets the address of the account that called the program.
    pub(crate) fn msg_sender(&self) -> [u8; 42] {
        self.msg_sender
    }

    pub(crate) fn store_event(&mut self, event: &[u8]) {
        self.events.push(Vec::from(event));
    }
}

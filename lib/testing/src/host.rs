//! The [`Host`] implementation that connects contracts to an [`Env`].
use std::slice;

use alloy_primitives::{keccak256, Address, B256, U256};
use stylus_sdk::{stylus_core::host::*, testing::TestVM};

use crate::env::{CallKind, Env};

/// Host VM backed by an [`Env`].
///
/// Every contract instance created from this host reads and writes the
/// storage of the contract whose frame is executing (held by that contract's
/// [`TestVM`]), sees that frame's message, and reaches other contracts of the
/// same environment through real calls. Balances, code and the block context
/// come from the environment's shared [`TestVM`].
#[derive(Clone)]
pub struct EnvHost {
    pub(crate) env: Env,
}

impl EnvHost {
    /// The environment behind this host.
    #[must_use]
    pub fn env(&self) -> &Env {
        &self.env
    }

    fn chain(&self) -> TestVM {
        self.env.state.borrow().chain.clone()
    }

    fn this(&self) -> Address {
        self.env.state.borrow().frame().this
    }

    /// The test VM of the executing contract.
    fn frame_vm(&self) -> TestVM {
        let this = self.this();
        self.env.vm_of(this).unwrap_or_else(|| {
            panic!("no contract is deployed at {}", self.env.name_of(this))
        })
    }

    /// Performs a call and writes the result length into `outs_len`.
    ///
    /// Returns `0` on success and `1` on revert, like the host I/O.
    fn perform_call(
        &self,
        kind: CallKind,
        to: Address,
        data: &[u8],
        value: U256,
        outs_len: &mut usize,
    ) -> u8 {
        match self.env.call(kind, to, data.to_vec(), value) {
            Ok(data) => {
                *outs_len = data.len();
                0
            }
            Err(data) => {
                *outs_len = data.len();
                1
            }
        }
    }

    fn record_log(&self, topics: Vec<B256>, data: Vec<u8>) {
        let mut state = self.env.state.borrow_mut();
        let emitter = state.frame().this;
        state.logs.push(crate::env::Log { emitter, topics, data });
    }
}

impl Host for EnvHost {}

impl CryptographyAccess for EnvHost {
    fn native_keccak256(&self, input: &[u8]) -> B256 {
        keccak256(input)
    }
}

impl CalldataAccess for EnvHost {
    fn read_args(&self, _len: usize) -> Vec<u8> {
        unimplemented!("calldata is passed to the router directly")
    }

    fn read_return_data(&self, offset: usize, size: Option<usize>) -> Vec<u8> {
        let state = self.env.state.borrow();
        let data = &state.return_data;
        let start = offset.min(data.len());
        let end = size.map_or(data.len(), |s| (start + s).min(data.len()));
        data[start..end].to_vec()
    }

    fn return_data_size(&self) -> usize {
        self.env.state.borrow().return_data.len()
    }

    fn write_result(&self, data: &[u8]) {
        self.env.state.borrow_mut().return_data = data.to_vec();
    }
}

unsafe impl UnsafeDeploymentAccess for EnvHost {
    unsafe fn create1(
        &self,
        code: *const u8,
        code_len: usize,
        endowment: *const u8,
        contract: *mut u8,
        revert_data_len: *mut usize,
    ) {
        // Deployments are answered by the mocks of the executing contract's
        // test VM (`TestVM::mock_deploy`).
        self.frame_vm().create1(
            code,
            code_len,
            endowment,
            contract,
            revert_data_len,
        );
    }

    unsafe fn create2(
        &self,
        code: *const u8,
        code_len: usize,
        endowment: *const u8,
        salt: *const u8,
        contract: *mut u8,
        revert_data_len: *mut usize,
    ) {
        self.frame_vm().create2(
            code,
            code_len,
            endowment,
            salt,
            contract,
            revert_data_len,
        );
    }
}

impl StorageAccess for EnvHost {
    fn storage_load_bytes32(&self, key: U256) -> B256 {
        self.frame_vm().get_storage(key)
    }

    unsafe fn storage_cache_bytes32(&self, key: U256, value: B256) {
        let mut state = self.env.state.borrow_mut();
        let frame = state.frames.last_mut().expect("no executing frame");
        if frame.is_static {
            frame.static_violation = true;
            return;
        }
        let this = frame.this;
        let vm = state
            .contracts
            .get(&this)
            .map(|deployed| deployed.vm.clone())
            .expect("the executing contract should be deployed");
        drop(state);
        vm.set_storage(key, value);
    }

    fn flush_cache(&self, _clear: bool) {}
}

unsafe impl UnsafeCallAccess for EnvHost {
    unsafe fn call_contract(
        &self,
        to: *const u8,
        data: *const u8,
        data_len: usize,
        value: *const u8,
        _gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        let to = Address::from_slice(slice::from_raw_parts(to, 20));
        let data = slice::from_raw_parts(data, data_len);
        let value = U256::from_be_slice(slice::from_raw_parts(value, 32));
        self.perform_call(CallKind::Call, to, data, value, outs_len)
    }

    unsafe fn static_call_contract(
        &self,
        to: *const u8,
        data: *const u8,
        data_len: usize,
        _gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        let to = Address::from_slice(slice::from_raw_parts(to, 20));
        let data = slice::from_raw_parts(data, data_len);
        self.perform_call(CallKind::StaticCall, to, data, U256::ZERO, outs_len)
    }

    unsafe fn delegate_call_contract(
        &self,
        to: *const u8,
        data: *const u8,
        data_len: usize,
        _gas: u64,
        outs_len: &mut usize,
    ) -> u8 {
        let to = Address::from_slice(slice::from_raw_parts(to, 20));
        let data = slice::from_raw_parts(data, data_len);
        self.perform_call(
            CallKind::DelegateCall,
            to,
            data,
            U256::ZERO,
            outs_len,
        )
    }
}

impl BlockAccess for EnvHost {
    fn block_basefee(&self) -> U256 {
        self.chain().block_basefee()
    }

    fn block_coinbase(&self) -> Address {
        self.chain().block_coinbase()
    }

    fn block_number(&self) -> u64 {
        self.chain().block_number()
    }

    fn block_timestamp(&self) -> u64 {
        self.chain().block_timestamp()
    }

    fn block_gas_limit(&self) -> u64 {
        self.chain().block_gas_limit()
    }
}

impl ChainAccess for EnvHost {
    fn chain_id(&self) -> u64 {
        self.chain().chain_id()
    }
}

impl AccountAccess for EnvHost {
    fn balance(&self, account: Address) -> U256 {
        self.chain().balance(account)
    }

    fn contract_address(&self) -> Address {
        self.this()
    }

    fn code(&self, account: Address) -> Vec<u8> {
        self.chain().code(account)
    }

    fn code_size(&self, account: Address) -> usize {
        self.chain().code_size(account)
    }

    fn code_hash(&self, account: Address) -> B256 {
        self.chain().code_hash(account)
    }
}

impl MemoryAccess for EnvHost {
    fn pay_for_memory_grow(&self, _pages: u16) {}
}

impl MessageAccess for EnvHost {
    fn msg_sender(&self) -> Address {
        self.env.state.borrow().frame().sender
    }

    fn msg_reentrant(&self) -> bool {
        let state = self.env.state.borrow();
        let this = state.frame().this;
        state.frames[..state.frames.len() - 1]
            .iter()
            .any(|frame| frame.this == this)
    }

    fn msg_value(&self) -> U256 {
        self.env.state.borrow().frame().value
    }

    fn tx_origin(&self) -> Address {
        // The shared VM's sender is set to the sender of the top-level call,
        // which is the default origin.
        self.chain().tx_origin()
    }
}

impl MeteringAccess for EnvHost {
    fn evm_gas_left(&self) -> u64 {
        self.chain().evm_gas_left()
    }

    fn evm_ink_left(&self) -> u64 {
        self.chain().evm_ink_left()
    }

    fn tx_gas_price(&self) -> U256 {
        self.chain().tx_gas_price()
    }

    fn tx_ink_price(&self) -> u32 {
        self.chain().tx_ink_price()
    }
}

impl RawLogAccess for EnvHost {
    fn emit_log(&self, input: &[u8], num_topics: usize) {
        let (topics, data) = input.split_at(num_topics * 32);
        let topics = topics.chunks(32).map(B256::from_slice).collect();
        self.record_log(topics, data.to_vec());
    }

    fn raw_log(
        &self,
        topics: &[B256],
        data: &[u8],
    ) -> Result<(), &'static str> {
        self.record_log(topics.to_vec(), data.to_vec());
        Ok(())
    }
}

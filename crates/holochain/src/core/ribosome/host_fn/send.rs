use crate::core::ribosome::error::RibosomeResult;
use crate::core::ribosome::wasm_ribosome::WasmRibosome;
use crate::core::ribosome::HostContext;
use holochain_zome_types::SendInput;
use holochain_zome_types::SendOutput;
use std::sync::Arc;

pub async fn send(
    _ribosome: Arc<WasmRibosome>,
    _host_context: Arc<HostContext>,
    _input: SendInput,
) -> RibosomeResult<SendOutput> {
    unimplemented!();
}
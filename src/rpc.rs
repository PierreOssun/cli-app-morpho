use crate::types::{CliError, Read};
use alloy::dyn_abi::{DynSolType, DynSolValue, FunctionExt, JsonAbiExt, Specifier};
use alloy::json_abi::Function;
use alloy::primitives::Address;
use alloy::providers::{DynProvider, Provider};
use alloy::rpc::types::TransactionRequest;

/// Calls `signature` (cast format, e.g. `totalAssets()(uint256)`) on `to` at `block`.
///
/// # Errors
/// Bad signature, bad args, revert or RPC failure.
pub async fn call(provider: &DynProvider, name: &str, to: Address, signature: &str, args: &[String], block: u64) -> Result<Read, CliError> {
    let function = Function::parse(signature)?;
    let values = function.inputs.iter().zip(args).map(|(input, arg)| Ok(input.resolve()?.coerce_str(arg)?)).collect::<Result<Vec<_>, CliError>>()?;
    let request = TransactionRequest::default().to(to).input(function.abi_encode_input(&values)?.into());
    let output = provider.call(request).block(block.into()).await?;
    Ok(Read {
        name: name.into(),
        to,
        call: signature.into(),
        args: args.to_vec(),
        block,
        result: function.abi_decode_output(&output)?.iter().map(text).collect(),
    })
}

/// A decoded value as a plain string.
fn text(value: &DynSolValue) -> String {
    match value {
        DynSolValue::Address(address) => address.to_string(),
        DynSolValue::Uint(uint, _) => uint.to_string(),
        DynSolValue::Bool(flag) => flag.to_string(),
        DynSolValue::String(string) => string.clone(),
        DynSolValue::FixedBytes(word, _) => word.to_string(),
        DynSolValue::Bytes(bytes) => alloy::hex::encode_prefixed(bytes),
        DynSolValue::Tuple(fields) => fields.iter().map(text).collect::<Vec<_>>().join(","),
        other => format!("{other:?}"),
    }
}

/// ABI-decodes hex `data` as `ty` (e.g. `(address,uint256)`), one string per field.
///
/// # Errors
/// Bad type, or data that does not decode as it.
pub fn decode(ty: &str, data: &str) -> Result<Vec<String>, CliError> {
    match DynSolType::parse(ty)?.abi_decode(&alloy::hex::decode(data)?)? {
        DynSolValue::Tuple(fields) => Ok(fields.iter().map(text).collect()),
        value => Ok(vec![text(&value)]),
    }
}

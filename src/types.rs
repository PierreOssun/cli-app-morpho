use alloy::primitives::{Address, U256};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Display;

/// The app's only error.
#[derive(Debug)]
pub struct CliError(pub String);

impl<E: Display> From<E> for CliError {
    fn from(e: E) -> Self {
        CliError(e.to_string())
    }
}

/// Chain state shared by every fact group.
pub struct Context {
    pub chain_id: u64,
    pub block: u64,
    pub block_timestamp: u64,
    pub block_30d: u64,
    pub block_30d_timestamp: u64,
    pub block_90d: u64,
    pub block_90d_timestamp: u64,
}

/// The evidence.json file.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceJson {
    pub schema: &'static str,
    pub chain_id: u64,
    pub block: u64,
    pub block_timestamp: u64,
    pub blocks: BTreeMap<&'static str, u64>,
    pub vaults: BTreeMap<&'static str, VaultJson>,
}

/// One vault's reads and the facts built from them.
#[derive(Serialize)]
pub struct VaultJson {
    pub address: Address,
    pub reads: Vec<Read>,
    pub facts: Vec<FactJson>,
}

/// One evidence fact. Fields that do not apply to its kind are left out of the JSON.
#[derive(Serialize, Default)]
pub struct FactJson {
    pub id: String,
    pub status: &'static str,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Read>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl FactJson {
    /// A fact read directly from one call.
    pub fn rpc(id: &str, source: &Read, decimals: Option<u8>, unit: Option<&str>, display: String) -> Self {
        FactJson {
            id: id.into(),
            status: "ok",
            kind: "rpc",
            value: Some(source.result[0].clone().into()),
            decimals,
            unit: unit.map(Into::into),
            display: Some(display),
            source: Some(source.clone()),
            ..Default::default()
        }
    }

    /// A fact computed from the reads named in `inputs`.
    pub fn derived(id: &str, value: impl Into<Value>, unit: &str, display: String, formula: &str, inputs: Vec<String>) -> Self {
        FactJson {
            id: id.into(),
            status: "ok",
            kind: "derived",
            value: Some(value.into()),
            unit: Some(unit.into()),
            display: Some(display),
            formula: Some(formula.into()),
            inputs: Some(inputs),
            ..Default::default()
        }
    }

    /// A fact that could not be established.
    pub fn error(id: &str, reason: &str) -> Self {
        FactJson { id: id.into(), status: "error", kind: "derived", reason: Some(reason.into()), ..Default::default() }
    }
}

/// One on-chain read and its result, one string per return value.
#[derive(Serialize, Clone)]
pub struct Read {
    pub name: String,
    pub to: Address,
    pub call: String,
    pub args: Vec<String>,
    pub block: u64,
    pub result: Vec<String>,
}

/// Every read made for one vault, each with a unique name.
pub struct Reads(pub Vec<Read>);

impl Reads {
    /// The read with this name, if it was made and succeeded.
    pub fn find(&self, name: &str) -> Option<&Read> {
        self.0.iter().find(|read| read.name == name)
    }

    /// The read with this name.
    ///
    /// # Errors
    /// The read is missing.
    pub fn read(&self, name: &str) -> Result<&Read, CliError> {
        Ok(self.find(name).ok_or(format!("missing read {name}"))?)
    }

    /// First return value of the read with this name.
    pub fn text(&self, name: &str) -> Result<&str, CliError> {
        Ok(&self.read(name)?.result[0])
    }

    /// First return value as an integer.
    pub fn uint(&self, name: &str) -> Result<U256, CliError> {
        Ok(self.text(name)?.parse()?)
    }

    /// First return value as f64, for ratios only: token amounts stay integers.
    pub fn float(&self, name: &str) -> Result<f64, CliError> {
        Ok(f64::from(self.uint(name)?))
    }
}

//! NEP-330 `contract_source_metadata`.
//!
//! `near-sdk` 4 does not emit this view. `cargo near` exports the same values
//! while it compiles, and this reads those so a verifier can see which image,
//! commit, and command produced the wasm.

use near_sdk::serde_json::{json, Value};

pub fn contract_source_metadata() -> Value {
    let version = option_env!("NEP330_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    let link = option_env!("NEP330_LINK").unwrap_or(env!("CARGO_PKG_REPOSITORY"));
    let mut metadata = json!({
        "version": version,
        "link": link,
        "standards": [{ "standard": "nep330", "version": "1.3.0" }],
    });

    let Some(build_environment) = option_env!("NEP330_BUILD_INFO_BUILD_ENVIRONMENT") else {
        return metadata;
    };
    if build_environment.is_empty() {
        return metadata;
    }

    let build_command = option_env!("NEP330_BUILD_INFO_BUILD_COMMAND")
        .and_then(|command| near_sdk::serde_json::from_str::<Value>(command).ok())
        .filter(|command| command.is_array())
        .unwrap_or_else(|| json!([]));
    let mut build_info = json!({
        "build_environment": build_environment,
        "build_command": build_command,
        "contract_path": option_env!("NEP330_BUILD_INFO_CONTRACT_PATH").unwrap_or(""),
        "source_code_snapshot": option_env!("NEP330_BUILD_INFO_SOURCE_CODE_SNAPSHOT").unwrap_or(""),
    });
    if let Some(output_wasm_path) = option_env!("NEP330_BUILD_INFO_OUTPUT_WASM_PATH") {
        if !output_wasm_path.is_empty() {
            build_info["output_wasm_path"] = json!(output_wasm_path);
        }
    }
    metadata["build_info"] = build_info;
    metadata
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_nep330_without_a_reproducible_build_stamp() {
        let metadata = super::contract_source_metadata();
        assert_eq!(metadata["standards"][0]["standard"], "nep330");
        assert_eq!(metadata["version"], "2.0.0");
        assert!(metadata["link"]
            .as_str()
            .unwrap()
            .contains("mpdao-vote-contracts"));
        assert!(metadata.get("build_info").is_none());
    }
}

use std::env;

fn env_flag(name: &str) -> bool {
    env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn maybe_patch_tauri_config_for_dev_check() {
    println!("cargo:rerun-if-env-changed=HONE_SKIP_BUNDLED_RESOURCE_CHECK");

    if !env_flag("HONE_SKIP_BUNDLED_RESOURCE_CHECK") {
        return;
    }

    let mut merged = env::var("TAURI_CONFIG")
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let bundle = merged
        .as_object_mut()
        .expect("tauri config patch root must be object")
        .entry("bundle")
        .or_insert_with(|| serde_json::json!({}));
    let bundle = bundle
        .as_object_mut()
        .expect("tauri config patch bundle must be object");
    bundle.insert("active".to_string(), serde_json::Value::Bool(false));
    bundle.insert(
        "externalBin".to_string(),
        serde_json::Value::Array(Vec::new()),
    );

    println!(
        "cargo:warning=HONE_SKIP_BUNDLED_RESOURCE_CHECK=1; skipping Tauri bundled sidecar validation for dev/IDE checks"
    );
    unsafe {
        env::set_var(
            "TAURI_CONFIG",
            serde_json::to_string(&merged).expect("serialize tauri config patch"),
        );
    }
}

fn main() {
    maybe_patch_tauri_config_for_dev_check();
    tauri_build::build()
}

use plugin_sdk::guest::{GuestPlugin, Host};
use plugin_sdk::PluginResult;
use plugin_sdk::guest_plugin;
use serde_json::{json, Value};
use std::sync::Mutex;

#[derive(Default)]
struct MyPlugin {
    calls: Mutex<u32>,
}

impl GuestPlugin for MyPlugin {
    fn descriptor(&self) -> Value {
        json!({
            "id": "my-plugin",
            "version": "1.0.0",
            "minHost": "0.1.0",
            "abi": plugin_sdk::ABI_VERSION,
            "displayName": "My Plugin",
            "summary": "参考插件，验证 packaged 插件可经插件 ABI 读取宿主能力",
            "capabilities": ["library.read", "audio.process"],
            "dependsOn": [],
            "optionalDependsOn": []
        })
    }

    fn activate(&self, _host: &Host) -> PluginResult<Value> {
        *self.calls.lock().unwrap() = 0;
        Ok(json!({
            "contributions": [
                { "kind": "command", "id": "my.inspect", "title": "Inspect library via host", "category": null, "inputSchema": null },
                { "kind": "command", "id": "my.eq", "title": "Read EQ bands via host", "category": null, "inputSchema": null }
            ],
            "subscriptions": [],
            "serviceIds": []
        }))
    }

    fn command(&self, host: &Host, command: &str, _args: &Value) -> PluginResult<Value> {
        *self.calls.lock().unwrap() += 1;
        match command {
            "my.inspect" => {
                let handle = host.resolve("library.read")?;
                let result = host.call(handle, "trackPath", &json!({ "trackId": 1 }))?;
                host.release(handle);
                Ok(json!({ "trackPath": result }))
            }
            "my.eq" => {
                let handle = host.resolve("audio.equalizer")?;
                let bands = host.call(handle, "getBands", &Value::Null)?;
                host.release(handle);
                Ok(json!({ "bands": bands }))
            }
            other => Err(plugin_sdk::PluginError::plugin(format!(
                "未知命令 '{other}'"
            ))),
        }
    }
}

guest_plugin!(MyPlugin);

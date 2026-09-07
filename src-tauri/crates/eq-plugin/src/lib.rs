use plugin_sdk::{
    capabilities, services, ActivationContext, CommandId, CommandSpec, Contribution, InvokeContext,
    Manifest, MenuSpec, NativeViewSpec, PluginDescriptor, PluginError, PluginId, PluginResult,
    PluginSettingsApi, Plugin, SidebarSpec, EqualizerApi, ServiceHandle, ServiceRef, SettingSpec,
    TeardownContext, Version,
};
use serde_json::{json, Value};

pub const ID: &str = "eq";

const VERSION: (u32, u32, u32) = (1, 0, 0);

const BAND_COUNT: usize = 10;

const DEFAULT_PRESET_NAME: &str = "Flat";

pub fn command_set_band() -> CommandId {
    CommandId::new("eq.setBand").expect("static command id is valid")
}
pub fn command_apply_preset() -> CommandId {
    CommandId::new("eq.applyPreset").expect("static command id is valid")
}
pub fn command_set_enabled() -> CommandId {
    CommandId::new("eq.setEnabled").expect("static command id is valid")
}
pub fn command_get_state() -> CommandId {
    CommandId::new("eq.getState").expect("static command id is valid")
}
pub fn command_reset() -> CommandId {
    CommandId::new("eq.reset").expect("static command id is valid")
}

pub fn manifest() -> Manifest {
    let mut manifest = Manifest::builtin(
        PluginId::new(ID).expect("eq id is valid"),
        Version::new(VERSION.0, VERSION.1, VERSION.2),
        Version::new(0, 1, 0),
    );
    manifest.name = ID.into();
    manifest.display_name = "均衡器".into();
    manifest.author = "RustEchoMusic".into();
    manifest.description = "10 段音频均衡器，支持预设与开关".into();
    manifest.capabilities = vec![capabilities::audio_process()];
    manifest.settings = vec![
        SettingSpec {
            key: "enabled".into(),
            title: "启用均衡器".into(),
            default_value: json!(true),
            control: "toggle".into(),
            options: None,
        },
        SettingSpec {
            key: "preset".into(),
            title: "频段预设".into(),
            default_value: json!([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            control: "eqBands".into(),
            options: None,
        },
        SettingSpec {
            key: "preset_name".into(),
            title: "预设名称".into(),
            default_value: json!(DEFAULT_PRESET_NAME),
            control: "text".into(),
            options: None,
        },
    ];
    manifest
}

pub struct EqPlugin {
    descriptor: PluginDescriptor,
}

impl EqPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: PluginDescriptor::new(
                PluginId::new(ID).expect("eq id is valid"),
                Version::new(VERSION.0, VERSION.1, VERSION.2),
                Version::new(0, 1, 0),
            )
            .with_display_name("均衡器"),
        }
    }

    fn audio(ctx: &InvokeContext) -> PluginResult<ServiceRef<ServiceHandle<dyn EqualizerApi>>> {
        ctx.require::<ServiceHandle<dyn EqualizerApi>>(&services::equalizer())
    }

    fn settings(
        ctx: &InvokeContext,
    ) -> PluginResult<ServiceRef<ServiceHandle<dyn PluginSettingsApi>>> {
        ctx.require::<ServiceHandle<dyn PluginSettingsApi>>(&services::plugin_settings())
    }

    fn handle_set_band(&self, args: Value, ctx: &InvokeContext) -> PluginResult<Value> {
        let band = read_usize(&args, "band")?;
        if band >= BAND_COUNT {
            return Err(PluginError::invalid_argument(format!(
                "EQ band index {band} out of range"
            )));
        }
        let gain = read_f64(&args, "gain")?;
        Self::audio(ctx)?.get()?.set_band_gain(band, gain)?;
        let bands = Self::audio(ctx)?.get()?.get_bands()?;
        Self::settings(ctx)?
            .get()?
            .set("preset", json!(bands))?;
        Ok(Value::Null)
    }

    fn handle_apply_preset(&self, args: Value, ctx: &InvokeContext) -> PluginResult<Value> {
        let bands = read_bands(&args, "bands")?;
        Self::audio(ctx)?.get()?.apply_preset(bands)?;
        Self::settings(ctx)?
            .get()?
            .set("preset", json!(bands))?;
        if let Some(name) = args.get("presetName").and_then(Value::as_str) {
            Self::settings(ctx)?
                .get()?
                .set("preset_name", json!(name))?;
        }
        Ok(Value::Null)
    }

    fn handle_set_enabled(&self, args: Value, ctx: &InvokeContext) -> PluginResult<Value> {
        let enabled = read_bool(&args, "enabled")?;
        Self::audio(ctx)?.get()?.set_enabled(enabled)?;
        Self::settings(ctx)?
            .get()?
            .set("enabled", json!(enabled))?;
        Ok(Value::Null)
    }

    fn handle_get_state(&self, ctx: &InvokeContext) -> PluginResult<Value> {
        let bands = Self::audio(ctx)?.get()?.get_bands()?;
        let enabled = Self::audio(ctx)?.get()?.is_enabled()?;
        let preset_value: Option<Value> = Self::settings(ctx)?.get()?.get("preset_name");
        let preset_name = preset_value
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| DEFAULT_PRESET_NAME.to_string());
        let state: Value = json!({
            "bands": bands,
            "enabled": enabled,
            "presetName": preset_name,
        });
        Ok(state)
    }

    fn handle_reset(&self, ctx: &InvokeContext) -> PluginResult<Value> {
        let bands: [f64; BAND_COUNT] = [0.0; BAND_COUNT];
        Self::audio(ctx)?.get()?.apply_preset(bands)?;
        Self::settings(ctx)?.get()?.set("preset", json!(bands))?;
        Ok(Value::Null)
    }
}

impl Default for EqPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for EqPlugin {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn activate(&self, ctx: &mut ActivationContext) -> PluginResult<()> {
        // 快速失败：没有音频服务就没必要登记命令，否则前端会拿到
        ctx.require::<ServiceHandle<dyn EqualizerApi>>(&services::equalizer())?;

        for (id, title) in [
            (command_set_band(), "设置 EQ 频段"),
            (command_apply_preset(), "应用 EQ 预设"),
            (command_set_enabled(), "切换 EQ 开关"),
            (command_get_state(), "读取 EQ 状态"),
            (command_reset(), "重置 EQ"),
        ] {
            ctx.contribute(Contribution::Command(CommandSpec {
                id,
                title: title.into(),
                category: Some("eq".into()),
                input_schema: None,
            }));
        }
        ctx.contribute(Contribution::SidebarItem(SidebarSpec {
            id: "eq".into(),
            title: "均衡器".into(),
            icon: "graphic_eq".into(),
            target: "eq-panel-view".into(),
        }));
        ctx.contribute(Contribution::NativeView(NativeViewSpec {
            id: "eq-panel-view".into(),
            title: "均衡器".into(),
            token: "eq-panel".into(),
            icon: Some("graphic_eq".into()),
        }));
        ctx.contribute(Contribution::MenuItem(MenuSpec {
            command: command_apply_preset(),
            title: "应用 EQ 预设".into(),
            location: "track.context".into(),
            group: Some("eq".into()),
            order: Some(10),
        }));
        Ok(())
    }

    fn deactivate(&self, _ctx: &TeardownContext) -> PluginResult<()> {
        // 无内存态：状态全部在全局 `AudioState` 与插件设置里。
        Ok(())
    }

    fn execute(&self, command: &CommandId, args: Value, ctx: &InvokeContext) -> PluginResult<Value> {
        match command.as_str() {
            "eq.setBand" => self.handle_set_band(args, ctx),
            "eq.applyPreset" => self.handle_apply_preset(args, ctx),
            "eq.setEnabled" => self.handle_set_enabled(args, ctx),
            "eq.getState" => self.handle_get_state(ctx),
            "eq.reset" => self.handle_reset(ctx),
            other => Err(PluginError::not_found(format!("eq 插件不处理命令 '{other}'"))),
        }
    }
}

fn read_usize(args: &Value, key: &str) -> PluginResult<usize> {
    match args.get(key) {
        Some(Value::Number(number)) => number
            .as_u64()
            .map(|v| v as usize)
            .ok_or_else(|| PluginError::invalid_argument(format!("'{key}' must be a non-negative integer"))),
        Some(Value::String(text)) => text
            .parse::<usize>()
            .map_err(|_| PluginError::invalid_argument(format!("'{key}' is not an integer"))),
        _ => Err(PluginError::invalid_argument(format!(
            "missing integer '{key}'"
        ))),
    }
}

fn read_f64(args: &Value, key: &str) -> PluginResult<f64> {
    match args.get(key) {
        Some(Value::Number(number)) => number
            .as_f64()
            .ok_or_else(|| PluginError::invalid_argument(format!("'{key}' must be a number"))),
        Some(Value::String(text)) => text
            .parse::<f64>()
            .map_err(|_| PluginError::invalid_argument(format!("'{key}' is not a number"))),
        _ => Err(PluginError::invalid_argument(format!(
            "missing number '{key}'"
        ))),
    }
}

fn read_bool(args: &Value, key: &str) -> PluginResult<bool> {
    match args.get(key) {
        Some(Value::Bool(value)) => Ok(*value),
        Some(Value::String(text)) => match text.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(PluginError::invalid_argument(format!(
                "'{key}' must be a boolean"
            ))),
        },
        _ => Err(PluginError::invalid_argument(format!(
            "missing boolean '{key}'"
        ))),
    }
}

fn read_bands(args: &Value, key: &str) -> PluginResult<[f64; BAND_COUNT]> {
    let array = args
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| PluginError::invalid_argument(format!("missing array '{key}'")))?;
    if array.len() != BAND_COUNT {
        return Err(PluginError::invalid_argument(format!(
            "'{key}' must have exactly {BAND_COUNT} entries, got {}",
            array.len()
        )));
    }
    let mut bands = [0.0; BAND_COUNT];
    for (index, item) in array.iter().enumerate() {
        bands[index] = item
            .as_f64()
            .ok_or_else(|| PluginError::invalid_argument(format!("'{key}'[{index}] not a number")))?;
    }
    Ok(bands)
}
use plugin_sdk::{
    capabilities, services, ActivationContext, CommandId, CommandSpec, Contribution, InvokeContext,
    Manifest, PluginDescriptor, PluginError, PluginId, PluginResult, PluginSettingsApi, Plugin,
    RecentReadApi, RecentWriteApi, ServiceHandle, ServiceRef, SettingSpec, TeardownContext, Version,
};
use serde_json::{json, Value};

pub const ID: &str = "recent";

const VERSION: (u32, u32, u32) = (1, 0, 0);

const KEY_MAX_RECORDS: &str = "max_records";

const DEFAULT_MAX_RECORDS: i64 = 100;

const MAX_PAGE_SIZE: i64 = 500;

pub fn command_list() -> CommandId {
    CommandId::new("recent.list").expect("static command id is valid")
}
pub fn command_add() -> CommandId {
    CommandId::new("recent.add").expect("static command id is valid")
}
pub fn command_clear() -> CommandId {
    CommandId::new("recent.clear").expect("static command id is valid")
}

pub fn manifest() -> Manifest {
    let mut manifest = Manifest::core(
        PluginId::new(ID).expect("recent id is valid"),
        Version::new(VERSION.0, VERSION.1, VERSION.2),
        Version::new(0, 1, 0),
    );
    manifest.name = ID.into();
    manifest.display_name = "最近播放".into();
    manifest.author = "RustEchoMusic".into();
    manifest.description = "记录并展示最近播放过的曲目".into();
    manifest.capabilities = vec![capabilities::recent_read(), capabilities::recent_write()];
    manifest.settings = vec![SettingSpec {
        key: KEY_MAX_RECORDS.into(),
        title: "最多保留条数".into(),
        default_value: json!(DEFAULT_MAX_RECORDS),
        control: "number".into(),
        options: None,
    }];
    manifest
}

pub struct RecentPlugin {
    descriptor: PluginDescriptor,
}

impl RecentPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: PluginDescriptor::new(
                PluginId::new(ID).expect("recent id is valid"),
                Version::new(VERSION.0, VERSION.1, VERSION.2),
                Version::new(0, 1, 0),
            )
            .with_display_name("最近播放"),
        }
    }

    fn reader(ctx: &InvokeContext) -> PluginResult<ServiceRef<ServiceHandle<dyn RecentReadApi>>> {
        ctx.require::<ServiceHandle<dyn RecentReadApi>>(&services::recent_read())
    }

    fn writer(ctx: &InvokeContext) -> PluginResult<ServiceRef<ServiceHandle<dyn RecentWriteApi>>> {
        ctx.require::<ServiceHandle<dyn RecentWriteApi>>(&services::recent_write())
    }

    fn max_records(ctx: &InvokeContext) -> i64 {
        let Ok(reference) =
            ctx.require::<ServiceHandle<dyn PluginSettingsApi>>(&services::plugin_settings())
        else {
            return DEFAULT_MAX_RECORDS;
        };
        let Ok(settings) = reference.get() else {
            return DEFAULT_MAX_RECORDS;
        };
        settings
            .get(KEY_MAX_RECORDS)
            .and_then(|value| value.as_i64())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_RECORDS)
    }

    fn handle_list(&self, args: Value, ctx: &InvokeContext) -> PluginResult<Value> {
        let limit = read_i64(&args, "limit").unwrap_or(50).clamp(1, MAX_PAGE_SIZE);
        let offset = read_i64(&args, "offset").unwrap_or(0).max(0);
        Self::reader(ctx)?.get()?.list(limit, offset)
    }

    fn handle_add(&self, args: Value, ctx: &InvokeContext) -> PluginResult<Value> {
        let track_id = read_i64(&args, "trackId")
            .ok_or_else(|| PluginError::invalid_argument("recent.add 需要 trackId"))?;
        let played_at = args
            .get("playedAt")
            .and_then(Value::as_str)
            .ok_or_else(|| PluginError::invalid_argument("recent.add 需要 playedAt"))?;

        Self::writer(ctx)?.get()?.upsert(track_id, played_at)?;

        // 裁剪策略：**上限属于插件，不属于宿主**。
        // 宿主只提供 count / removeOldest 原语，保留多少条由这里决定。
        let cap = Self::max_records(ctx);
        let reader = Self::reader(ctx)?;
        if reader.get()?.count()? > cap {
            Self::writer(ctx)?.get()?.remove_oldest(cap)?;
        }
        Ok(Value::Null)
    }

    fn handle_clear(&self, ctx: &InvokeContext) -> PluginResult<Value> {
        Self::writer(ctx)?.get()?.clear()?;
        Ok(Value::Null)
    }
}

impl Default for RecentPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for RecentPlugin {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn activate(&self, ctx: &mut ActivationContext) -> PluginResult<()> {
        // 快速失败：宿主没注册存储服务就没必要把命令登记出去，
        // 否则前端会拿到"命令存在但每次都失败"这种最难排查的形态。
        ctx.require::<ServiceHandle<dyn RecentReadApi>>(&services::recent_read())?;
        ctx.require::<ServiceHandle<dyn RecentWriteApi>>(&services::recent_write())?;

        for (id, title) in [
            (command_list(), "读取最近播放"),
            (command_add(), "记录一次播放"),
            (command_clear(), "清空最近播放"),
        ] {
            ctx.contribute(Contribution::Command(CommandSpec {
                id,
                title: title.into(),
                category: Some("recent".into()),
                input_schema: None,
            }));
        }
        ctx.contribute(Contribution::Setting(SettingSpec {
            key: KEY_MAX_RECORDS.into(),
            title: "最多保留条数".into(),
            default_value: json!(DEFAULT_MAX_RECORDS),
            control: "number".into(),
            options: None,
        }));
        Ok(())
    }

    fn deactivate(&self, _ctx: &TeardownContext) -> PluginResult<()> {
        // 无内存态：记录全部落在宿主的 SQLite 里，没什么要 flush 的。
        Ok(())
    }

    fn execute(&self, command: &CommandId, args: Value, ctx: &InvokeContext) -> PluginResult<Value> {
        match command.as_str() {
            "recent.list" => self.handle_list(args, ctx),
            "recent.add" => self.handle_add(args, ctx),
            "recent.clear" => self.handle_clear(ctx),
            other => Err(PluginError::not_found(format!(
                "recent 插件不处理命令 '{other}'"
            ))),
        }
    }
}

fn read_i64(args: &Value, key: &str) -> Option<i64> {
    match args.get(key)? {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}
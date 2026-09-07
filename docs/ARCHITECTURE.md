# RustEchoMusic 架构说明

---

## 1. 系统概览

RustEchoMusic 是一个基于 **Tauri 2 + Rust** 的桌面音乐播放器，前端使用
**SvelteKit 5**。其核心特征是一条贯穿始终的设计原则——**"一切皆插件"**：
宿主自身提供的能力、以及第三方/内置插件对外暴露的能力，走的是同一套
生命周期、依赖注入、能力授权与贡献点机制。新增宿主能力 = 注册一个服务；
新增扩展点 = 增加一个贡献点；二者都不需要改动宿主主体代码。

### 技术栈

| 层 | 技术 |
|---|---|
| 应用壳 / 系统交互 | Tauri 2（tray-icon、dialog、opener、store 插件） |
| 核心语言 | Rust 2021（edition） |
| 音频 | `web-audio-api` 1.7（实时处理）、`lofty` 0.25（元数据）、`souvlaki` 0.8（媒体键） |
| 持久化 | `sqlx` 0.9 + SQLite（迁移脚本在 `src-tauri/migrations/`） |
| 动态加载 | `libloading` 0.9（dylib 插件） |
| 前端 | SvelteKit 2、Svelte 5（runes）、Tailwind CSS 4、MDUI 2 |
| 构建 | Vite 8 + `@sveltejs/vite-plugin-svelte` 7、TypeScript 6 |
| 包管理 | pnpm 11 |

### 分层

```
┌──────────────────────────────────────────────────────────────────┐
│ 前端（SvelteKit）                                                  │
│  routes / features / state  ·  lib/plugins（host-bridge 桥接层）     │
├───────────────────────────────┬──────────────────────────────────┤
│ Tauri 命令层（commands/）       │ WebView（经 @tauri-apps/api invoke）│
├───────────────────────────────┴──────────────────────────────────┤
│ 宿主（src-tauri）                                                 │
│  · 业务服务：playback / track / settings / plugin-kernel           │
│  · 消费贡献点：菜单、命令、原生视图、侧边栏、音频处理器             │
├──────────────────────────────────────────────────────────────────┤
│ plugin-runtime  （编排与策略：宿主唯一入口）                        │
│  lifecycle · container · contributions · discovery · deps          │
│  · supervisor（熔断）· loader · dynamic                            │
├──────────────────────────────────────────────────────────────────┤
│ plugin-sdk  （契约，宿主与插件共同依赖，不含实现）                  │
│  ids · error · event · service · plugin · manifest · abi          │
├──────────────────────────────────────────────────────────────────┤
│ 插件                                                              │
│  内置 crate（静态链接）      packaged dylib / iframe 插件（运行时加载）│
│        └──────────── 同一个 Plugin trait ────────────┘             │
└──────────────────────────────────────────────────────────────────┘
```

**依赖方向严格单向**：插件 → SDK → （无反向依赖）。`plugin-sdk` 不依赖
`plugin-runtime`，因此插件编译时不会被拖进宿主代码。

---

## 2. 仓库与模块布局

```
RustEchoMusic/
├── src-tauri/                     # Rust 宿主（Tauri 2）
│   ├── Cargo.toml                 # 工作区：app + 5 个插件/SDK crate
│   ├── crates/
│   │   ├── plugin-sdk/            # 插件契约（无实现）
│   │   ├── plugin-runtime/        # 生命周期/容器/贡献点/熔断/加载
│   │   ├── lyrics-plugin/         # 内置：歌词
│   │   ├── eq-plugin/             # 内置：均衡器
│   │   └── recent-plugin/         # 内置：最近播放
│   ├── src/
│   │   ├── lib.rs                 # Tauri 入口、setup、应用服务接线
│   │   ├── commands/              # Tauri 命令（含 plugin.rs / plugin_kernel.rs）
│   │   ├── services/
│   │   │   ├── playback_service.rs / track_service.rs / settings_service.rs
│   │   │   └── plugin/kernel/     # 宿主内核：host services + per-plugin 工厂 + 内置 seed
│   │   ├── models/  repositories/  # 领域模型 + sqlx 仓储
│   │   └── state/                 # 应用级状态（AppState 持有 PluginRuntime）
│   └── migrations/                # sqlx 迁移
├── src/                           # SvelteKit 前端
│   ├── routes/  features/  state/
│   ├── lib/plugins/               # host-bridge.ts / register-builtin-views.ts / component-registry.ts
│   ├── lib/types/                # 与宿主 wire 契约对齐的 TS 类型
│   └── lib/state/plugins.svelte.ts
└── docs/                          # 设计/迁移/审计文档
```

---

## 3. 插件系统（核心）

### 3.1 统一插件接口 `Plugin`

所有插件（内置 crate 或运行时加载的 dylib/iframe）实现同一个 trait：

```rust
pub trait Plugin: Send + Sync {
    fn descriptor(&self) -> &PluginDescriptor;
    fn activate(&self, ctx: &mut ActivationContext) -> PluginResult<()>;
    fn deactivate(&self, ctx: &TeardownContext) -> PluginResult<()>;

    // 以下均有默认实现：不关心即零样板
    fn on_event(&self, event: &HostEvent, ctx: &InvokeContext) -> PluginResult<()> { Ok(()) }
    fn execute(&self, cmd: &CommandId, args: Value, ctx: &InvokeContext) -> PluginResult<Value> { … }
    fn health(&self) -> HealthStatus { HealthStatus::Healthy }
    fn audio_processor(&self) -> Option<AudioProcessorHandle> { None }
}
```

设计取舍：`activate`/`deactivate` 之外的能力都是可选默认实现；
`audio_processor` 作为 trait 一等能力而非 downcast，内置插件同样可提供实时处理器。

### 3.2 强类型 id 与版本兼容

`ids.rs` 定义 6 个 newtype：`PluginId` / `ServiceId` / `ContributionPointId` /
`CommandId` / `Capability` / `EventType`（均 `serde(try_from = "String")`）。
非法字符串在**反序列化时**即失败，避免脏 id 混进注册表。

`Version::is_compatible_with()` 语义：主版本必须相同且不大于宿主，与 Cargo/semver
的 caret 语义一致。`minHost` 字段是兼容性判定唯一依据。

### 3.3 服务容器与依赖注入

宿主与插件注册/解析服务的接口**完全一致**：

```rust
// 宿主（kernel/mod.rs）
runtime.provide_host_service(ServiceDescriptor::new(services::player(), v),
    Arc::new(ServiceHandle::new(Arc::clone(&player) as Arc<dyn PlayerControlApi>)))?;

// 插件（激活期）
let player = ctx.require::<ServiceHandle<dyn PlayerControlApi>>(&services::player())?;
```

解析结果包在 `ServiceRef<T>` 中，持有**可撤销租约**（`ServiceSlot`）：

```rust
pub struct ServiceRef<T: ?Sized> { slot: Arc<ServiceSlot>, value: Arc<T> }
impl<T: ?Sized> ServiceRef<T> {
    pub fn get(&self) -> PluginResult<&T> {
        if self.slot.is_alive() { Ok(&self.value) }
        else { Err(PluginError::service_unavailable("...")) }
    }
}
```

这是**失败隔离的关键**：动态库 `dlclose` 之后访问其代码/数据是 UB。插件不持有裸
`Arc<T>`，而是持有 `ServiceRef<T>`，每次访问前先检查租约；提供方被卸载 → 租约失效 →
`get()` 报错而非踩空。

**注册类型必须匹配请求类型**：per-plugin 工厂的泛型 `T` 必须是插件侧
`require::<ServiceHandle<dyn Trait>>` 请求的同款特征对象，否则 `resolve_for` 中
TypeId 比对失败（详见 §3.12 的坑）。`HostPluginStorage` 等宿主实现在工厂里包成
`ServiceHandle::new(Arc::new(HostXxx::new(..)) as Arc<dyn Trait>)` 后返回。

### 3.4 授权三道闸

1. `grant(plugin, capabilities)` —— 激活时按清单**覆盖式**下发（非追加，避免残留旧授权）。
2. `check_visible()` —— `ServiceVisibility::{Public, DeclaredDependentsOnly, HostOnly}`。
3. `check_capabilities()` —— 服务的 `requires` 必须是请求方已获授权的子集。

动态库插件走 JSON 通道时，用 `authorize()` 做**同三道闸**的鉴权（无 Rust 类型信息）。

### 3.5 能力模型（Capability / ACL）

能力是字符串化的权限单元（`Capability` newtype），例如 `player.read`、
`storage.private`、`audio.process`。服务在注册时声明 `.requiring(capability)`，
插件在 `manifest.capabilities` 中静态声明所需能力；运行期激活时按清单覆盖式授权。
运行期请求的未在清单中的能力 → `PermissionDenied`（且激活回滚）。

### 3.6 生命周期状态机

9 个状态，`can_transition()` 是唯一迁移规则表（`lifecycle.rs`）：

```
Discovered ──► Resolved ──► Loaded ──► Activating ──► Active
                 │           │  ▲          │            │
                 │           │  └──────────┼────────────┤
                 ▼           ▼             ▼            ▼
              Unloaded ◄── Stopped ◄── Deactivating   Failed
                 ▲            │                          │
                 └────────────┴──────────────────────────┘
```

约束：激活中不允卸载（热插拔安全前提）；非法迁移返回 `InvalidArgument` 且**不改状态**；
保留最近 32 条迁移历史供诊断面板使用；`Failed` 可从 Active/Activating/Deactivating 进入。

### 3.7 发现与定位

`PluginLocator` 三种实现：`BuiltinLocator`（清单由代码提供）、`DirectoryLocator`
（扫盘）、`CompositeLocator`（聚合+去重）。去重规则：**内置插件优先于目录同名插件**，
防止"用户目录塞一个 `lyrics` 顶掉内置实现"的提权。

### 3.8 加载：内置 vs 运行时

- **内置插件**：作为 crate 静态链入，经 `runtime.with_builtin(manifest, factory)`
  登记；不可被用户禁用（core tier）。
- **packaged / user 插件（Rust dylib）**：`loader.rs` 用 `libloading` 打开，
  校验导出符号 `REM_PLUGIN_ABI` 与 `abi.is_compatible()`（任何调用之前），
  由 `Arc<Library>` 引用计数门控 `dlclose`。跨 ABI 通道为 JSON over C ABI，
  服务句柄编码为奇数以区别于宿主根上下文（偶数），避免解引用混淆；缓冲区遵循
  "谁分配谁释放"。
- **前端 iframe 插件**：沙箱 `<iframe>` 加载，经 `host-bridge.ts` 的 postMessage
  桥与宿主通信（见 §5）。

### 3.9 贡献点（Contribution）与扩展点

插件在激活期通过 `ctx.contribute(...)` 声明对外暴露物，宿主负责渲染：

```rust
ctx.contribute(Contribution::Command(CommandSpec { … }));
ctx.contribute(Contribution::MenuItem(MenuSpec { … }));
ctx.contribute(Contribution::AudioProcessor(AudioProcessorSpec { order, .. }));
ctx.contribute(Contribution::Extension { point, payload }); // 逃生口：新增扩展点不必发新 ABI
```

`Extension` 变体是刻意留的扩展点逃生口。`ContributionRegistry::apply()` 为
**全量替换**（非追加），保证热重载不残留旧贡献。

### 3.10 命令与事件派发

- 命令：`runtime.execute(&command, args)` 路由到 owner 插件；owner 未激活则**懒激活**。
  前端经 Tauri 命令 `execute_plugin_command` 触发。
- 事件：`runtime.emit(kind, payload)` / `dispatch(event)`，按订阅剪枝派发；
  单个插件失败只进 `failed`，不阻断其他插件；事件不回灌产生方。
- `DispatchStats { delivered, failed, skipped }` 供派发调优。

### 3.11 失败隔离（Supervisor）

| 风险 | 机制 | 错误码 |
|---|---|---|
| panic | `catch_unwind` | `Panic` |
| 超时 | 独立线程 + 看门狗 | `Timeout` |
| 连续失败 | `CircuitBreaker` 三态 | `ServiceUnavailable`（被拒） |

- 熔断参数来自清单 `budget`（调用超时 / 连续失败次数 / 冷却）。音频处理器用更紧的
  `realtime()` 预算；普通插件用默认值。
- 超时线程被 **detach**（Rust 无安全杀线程手段），契约是"宿主不再等它"——插件须遵守
  "调用不得长时间阻塞"。
- 级联：`CascadePolicy::StopDependents`（默认）下，熔断开启会停掉所有依赖者；
  `DependencyGraph::unload_impact()` 提供传递闭包，UI 可提前提示影响面。
- 实时音频额外保护：宿主在调用 `process` 前把 `output` 预填为 `input` 拷贝（插件不写
  数据也不静音，返回非 0 则保留预填=旁路）；`audio_processors()` 仅返回**当前激活**
  插件的处理器，停用即旁路。

### 3.12 热插拔

`deactivate` → `reload`（停用→卸载→重激活）→ `uninstall`（依赖者先停、被依赖者后停）。
停用顺序**刻意**为：先撤销外部可见物（`contribution.revoke` / `revoke_provider` /
`json.remove_provider` / `revoke_grants`），**后**调用插件 `deactivate()`——即使插件
卡住或 panic，宿主已处于安全态。

### 3.13 编排层 `PluginRuntime`

宿主通过 `PluginRuntime` 单例编排：

```rust
let runtime = PluginRuntime::new(host_version)
    .with_cascade(CascadePolicy::StopDependents)
    .with_builtin(lyrics_manifest, || Ok(Box::new(LyricsPlugin::new())))
    .with_builtin(eq_manifest,     || Ok(Box::new(EqPlugin::new())))
    .with_locator(DirectoryLocator::new(packaged_dir, PluginSource::Packaged))
    .with_locator(DirectoryLocator::new(user_dir,     PluginSource::User));

runtime.discover();          // 尽力而为，返回 issues
runtime.activate_all();       // 按拓扑顺序，逐项成败不中断整体启动
runtime.emit(kind, payload);  // 事件派发
runtime.execute(&cmd, args);  // 命令路由（懒激活）
runtime.diagnostics();        // 完整快照，直接序列化给前端
runtime.shutdown();           // 退出时按依赖逆序停用
```

**三条不可违背的实现约束**：

1. 调用插件时**不持有 `entries` 写锁**（避免回调宿主时死锁）；每个阶段拆成
   「取锁取数据→释放→调插件→再取锁提交」。
2. 激活是**事务**：要么全部贡献与服务登记成功，要么全部撤销并置 `Failed`。
3. 诊断取数分两遍：锁内只拷清单字段，释放锁后再做健康自检（自检会派生线程再读
   `entries`，持锁在 Windows 写者优先 RwLock 上有真实死锁风险）。

---

## 4. 宿主服务清单

| ServiceId | 能力要求 | 说明 |
|---|---|---|
| `player` | `player.control` | 播放控制（强类型通道） |
| `player.state` | `player.read` | 播放状态（只读） |
| `queue` | `queue.read` | 播放队列 |
| `library` | `library.read` | 曲库查询 |
| `settings` | `settings.read` | 应用设置 |
| `recent.read` | `recent.read` | 最近播放（读原语） |
| `recent.write` | `recent.write` | 最近播放（写原语） |
| `equalizer` | `audio.process` | 10 段均衡器 |
| `plugin.settings` | （无） | 插件私有设置，**每插件一份** |
| `storage` | `storage.private` | 插件私有目录（data/cache），**每插件一份** |
| `events` | （无） | 插件事件出口 |

前 8 个为宿主级服务（`provide_host_service`）；后 3 个为 per-plugin 工厂
（`provide_host_factory`），身份由容器在解析时注入，**插件拿不到别人的那份**。

---

## 5. 前端桥接

前端通过 **iframe 沙箱 + postMessage 桥** 与插件通信，协议信封与
`static/plugin-sdk/plugin-host-sdk.js` 对齐：
`{ source: 'rem-plugin-host' | 'rem-plugin', type, id?, payload? }`。

`lib/plugins/host-bridge.ts` 职责：
- `attach(iframe, pluginId)`：绑定 iframe、校验来源（`e.source !== iframe.contentWindow` 拒收）、
  load 时发 `ready` 握手并附 capabilities。
- `#handleMessage`：把插件请求路由到对应 Tauri 命令或前端状态——`command`
  （→ `execute_plugin_command`）、`settings:get/set`（→ `get_plugin_settings` /
  `update_plugin_setting`）、`state:get`（读 `player` 快照）、`subscribe/unsubscribe`
  （事件订阅剪枝）。
- `broadcastEvent`：将后端 `global-app-event` 按订阅剪枝广播给各 iframe（trackChanged /
  playbackStateChanged / queueChanged / settingsChanged）。

UI 渲染侧：
- `register-builtin-views.ts` / `component-registry.ts`：遍历贡献点注册内置视图。
- `PluginNativeViewHost.svelte`：承载插件原生视图。
- `LyricsPanel.svelte` / `EqPanel.svelte`：消费 lyrics / eq 插件的贡献。
- `plugins.svelte.ts`：持有运行时快照（`runtime.diagnostics()`）供管理页渲染。

---

## 6. 数据持久化

- **应用数据**：`sqlx` + SQLite，迁移脚本在 `src-tauri/migrations/`
  （`0001_initial.sql`、`0002_extend_settings.sql`）。sqlx 把迁移 SHA-256 校验和存入
  `_sqlx_migrations` 表，迁移文件被改动后校验和不匹配会启动失败（此时需重置开发库）。
- **插件私有数据**：通过 `storage` 服务的 `private_dir()` / `cache_dir()` 落在
  `<app_data>/plugins/<id>/{data,cache}`，与用户插件发现目录（`extensions/`）刻意分离，
  避免扫描噪声。

---

## 7. 设计取舍与经验沉淀

- **per-plugin 工厂注册陷阱**：工厂泛型 `T` 必须注册为 `ServiceHandle<dyn Trait>`
  特征对象，绝不能注册具体类型——否则 TypeId 不匹配、问题推迟到运行期才暴露
  （`ServiceUnavailable: registered but not with the requested type`）。
- **激活事务性 / 调用插件不持锁 / 诊断两遍取数**：写入 `plugin-runtime` 实现约束，
  是 Windows RwLock 与回调死锁的真实经验。
- **能力静态化**：运行期授权与依赖声明均以清单为唯一来源，保证"清单即契约"。

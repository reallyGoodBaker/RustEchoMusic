# RustEchoMusic

[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte)](https://svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-6-3178C6?logo=typescript)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**RustEchoMusic** 是一个基于 **Tauri 2 + Rust + SvelteKit + Svelte 5** 构建的本地桌面音乐播放器。

项目的重点不仅是播放本地音乐，更是探索一种适合桌面应用的**插件化架构**：将播放器能力抽象为可发现、可依赖、可授权、可独立停用的服务和插件，使功能扩展尽可能与宿主核心解耦。

目前包含本地音乐库、播放器、播放队列、最近播放、歌词、均衡器等功能，并提供统一的 Rust Plugin SDK、插件运行时以及前端插件桥接机制。

> 项目仍处于持续开发阶段。插件 ABI、SDK API 和部分运行时行为可能发生变化。

## Features

### Playback

- 本地音乐库管理
- 播放、暂停、切歌与播放队列
- 最近播放记录
- 音频元数据读取
- 系统媒体控制
- 面向大型音乐库的虚拟列表渲染

### Plugin System

- 内置插件与外部插件使用统一的 `Plugin` 抽象
- `plugin-sdk` 提供宿主与插件之间的类型化契约
- 插件发现、依赖解析、加载、激活、停用和卸载由 runtime 统一管理
- 支持命令、菜单、侧栏、视图、设置、事件等 Contribution Points
- 插件通过服务容器访问宿主能力，并以 Capability 声明权限需求
- 支持内置、打包和动态插件
- 支持音频处理器作为插件能力接入实时音频链路

### Runtime

- 生命周期状态跟踪
- 依赖图与级联停用
- 调用监督与超时处理
- 连续失败熔断
- Panic / 错误传播隔离
- 停用时撤销插件贡献与事件订阅
- 卸载前按生命周期顺序释放插件资源

### Frontend

- Svelte 5 Runes
- TypeScript
- Tailwind CSS 4
- MDUI 2
- iframe 沙箱插件 UI
- `postMessage` 宿主通信桥

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                         SvelteKit UI                        │
│          Player · Library · Queue · Settings · Plugin UI    │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                           Host                               │
│              Player · Library · Queue · Settings             │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                      Plugin Runtime                          │
│  Discovery · Dependency Graph · Lifecycle · Service DI      │
│  Contributions · Supervisor · Built-in / Packaged / Dynamic │
└──────────────────────────────┬──────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────┐
│                         Plugin SDK                           │
│     Plugin · Service · Capability · Event · Contribution    │
│                         · ABI                                │
└─────────────────────────────────────────────────────────────┘
```

### Service Container

宿主将播放器、队列、音乐库、设置、均衡器和事件等能力注册到服务容器中。插件通过类型化的 `ServiceHandle` 访问宿主服务，而不是直接依赖具体实现。

### Capability

插件通过 Capability 声明所需的宿主能力，例如：

- `player.read`
- `player.control`
- `queue.read`
- `queue.write`
- `library.read`
- `settings.read`
- `settings.write`
- `audio.process`
- `plugin.ui`
- `storage.private`
- `fs.userGranted`
- `net.fetch`
- `recent.read`
- `recent.write`

Runtime 在插件激活和服务解析过程中处理能力检查，使插件依赖关系与权限边界保持显式。

### Contribution Points

插件可以通过 Contribution Points 向宿主扩展功能，包括：

- Command
- Menu
- Sidebar
- Native View
- Settings
- Audio Processor

### Lifecycle

插件生命周期由 runtime 统一管理：

```text
Discovered
    ↓
Resolved
    ↓
Loaded
    ↓
Activating
    ↓
Active
    ↓
Deactivating
    ↓
Unloaded
```

依赖图决定插件激活、停用、失败传播和卸载顺序，避免插件直接管理其他插件的生命周期。

## Built-in Plugins

| Plugin | Description |
| --- | --- |
| `lyrics` | 歌词搜索、加载、缓存与同步显示 |
| `eq` | 10 段均衡器与预设 |
| `recent` | 最近播放记录 |
| `audio-gain` | 示例音频插件，用于验证插件 ABI 与实时音频链路 |

插件通过 `plugin.json` 描述自身的能力需求、激活事件、命令、UI Contribution 和设置项。

## Plugin Development

Plugin SDK 位于：

```text
src-tauri/crates/plugin-sdk/
```

一个最小的 Rust 插件结构示例：

```rust
impl Plugin for MyPlugin {
    fn descriptor(&self) -> &PluginDescriptor {
        &self.descriptor
    }

    fn activate(&mut self, ctx: &mut ActivationContext) -> PluginResult<()> {
        let player = ctx.services.require::<PlayerControlApi>()?;

        ctx.contributions.command(
            CommandId::new("my-plugin.action"),
            move |_ctx| {
                player.pause()?;
                Ok(())
            },
        )?;

        Ok(())
    }

    fn deactivate(&mut self, _reason: DeactivateReason) -> PluginResult<()> {
        Ok(())
    }
}
```

外部插件通过 `plugin.json` 描述插件元数据和运行能力，例如：

```json
{
  "schema": 2,
  "id": "audio-gain",
  "name": "audio-gain",
  "displayName": "Audio Gain",
  "version": "1.0.0",
  "minHost": "1.0.0",
  "abi": 2,
  "entry": "audio-gain",
  "source": "packaged",
  "capabilities": ["audio.process"],
  "activation": {
    "eager": true
  }
}
```

示例插件：

```text
plugins/packaged/audio-gain/
plugins/packaged/my-plugin/
```

## Project Structure

```text
.
├── plugins/
│   ├── eq/
│   ├── lyrics/
│   ├── recent/
│   └── packaged/
│       ├── audio-gain/
│       └── my-plugin/
├── src/
├── src-tauri/
│   ├── src/
│   └── crates/
│       ├── plugin-sdk/
│       ├── plugin-runtime/
│       ├── lyrics-plugin/
│       ├── eq-plugin/
│       └── recent-plugin/
├── Cargo.toml
├── package.json
└── pnpm-workspace.yaml
```

## Tech Stack

| Area | Technology |
| --- | --- |
| Desktop Runtime | Tauri 2 |
| Core | Rust 2021 |
| Frontend | SvelteKit 2 + Svelte 5 |
| Language | TypeScript 6 |
| Styling | Tailwind CSS 4 + MDUI 2 |
| Audio | `web-audio-api`, `souvlaki` |
| Metadata | `lofty` |
| Persistence | SQLite + SQLx |
| Build | Vite 8 + Tauri CLI |
| Package Manager | pnpm 11 |

## Getting Started

### Prerequisites

- Rust toolchain with Cargo
- Node.js 20+
- pnpm 11
- Tauri 2 platform dependencies

Tauri prerequisites:

https://v2.tauri.app/start/prerequisites/

### Install

```bash
pnpm install
```

### Development

```bash
pnpm tauri dev
```

### Type Check

```bash
pnpm check
```

### Build

```bash
pnpm tauri build
```

## Design Goals

1. **Keep the host small** — 核心播放器只负责核心能力，不承载所有扩展逻辑。
2. **Make extensions explicit** — 插件能力、权限和贡献点通过 SDK 与 manifest 显式描述。
3. **Isolate failures** — 插件异常、超时和连续失败不应直接破坏宿主运行时。
4. **Separate contracts from implementations** — SDK 定义契约，runtime 管理执行，具体插件实现业务能力。
5. **Treat lifecycle as a first-class concern** — 插件发现、依赖、激活、停用和卸载均由统一生命周期模型管理。

## Status

当前核心架构与插件运行时已经完成重构，并采用 Cargo workspace 管理宿主、SDK 与插件。

项目仍在持续开发中。插件 SDK、ABI 以及部分 runtime API 尚未承诺长期兼容，正式插件生态的兼容策略将在 API 稳定后进一步明确。

## Contributing

欢迎通过 Issue 和 Pull Request 参与项目。

提交新的插件能力时，建议同时考虑：

- SDK / runtime 是否需要新增契约
- manifest 是否能够完整描述插件能力
- 插件是否保持与宿主核心的边界
- 失败和停用时是否能够正确释放资源
- 是否需要补充示例插件或文档

## License

This project is licensed under the [MIT License](LICENSE).

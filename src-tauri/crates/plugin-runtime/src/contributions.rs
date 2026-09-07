use std::collections::HashMap;
use std::sync::RwLock;

use plugin_sdk::{
    AudioProcessorSpec, CommandId, CommandSpec, Contribution, ContributionPointId, MenuSpec,
    NativeViewSpec, PluginId, SettingSpec, SidebarSpec, ViewSpec,
};

#[derive(Debug, Clone)]
pub struct ContributionRecord {
    pub plugin: PluginId,
    pub contribution: Contribution,
}

pub struct ContributionRegistry {
    buckets: RwLock<HashMap<ContributionPointId, Vec<ContributionRecord>>>,
}

impl ContributionRegistry {
    pub fn new() -> Self {
        Self {
            buckets: RwLock::new(HashMap::new()),
        }
    }

    pub fn apply(&self, plugin: &PluginId, items: Vec<Contribution>) {
        let Ok(mut lock) = self.buckets.write() else {
            return;
        };
        // 先清空该插件的所有贡献（跨所有贡献点）。
        for bucket in lock.values_mut() {
            bucket.retain(|record| &record.plugin != plugin);
        }
        for contribution in items {
            lock.entry(contribution.point())
                .or_default()
                .push(ContributionRecord {
                    plugin: plugin.clone(),
                    contribution,
                });
        }
        lock.retain(|_, bucket| !bucket.is_empty());
    }

    pub fn revoke(&self, plugin: &PluginId) {
        if let Ok(mut lock) = self.buckets.write() {
            for bucket in lock.values_mut() {
                bucket.retain(|record| &record.plugin != plugin);
            }
            lock.retain(|_, bucket| !bucket.is_empty());
        }
    }

    pub fn query(&self, point: &ContributionPointId) -> Vec<ContributionRecord> {
        let Ok(lock) = self.buckets.read() else {
            return Vec::new();
        };
        lock.get(point).cloned().unwrap_or_default()
    }

    // —— 强类型便捷查询 ——

    pub fn commands(&self) -> Vec<(PluginId, CommandSpec)> {
        self.typed(|c| c.as_command().cloned())
    }

    pub fn menu_items(&self, location: &str) -> Vec<(PluginId, MenuSpec)> {
        self.typed_filter(|c| match c.as_menu_item() {
            Some(spec) if spec.location == location => Some(spec.clone()),
            _ => None,
        })
    }

    pub fn sidebars(&self) -> Vec<(PluginId, SidebarSpec)> {
        self.typed(|c| c.as_sidebar_item().cloned())
    }

    pub fn views(&self) -> Vec<(PluginId, ViewSpec)> {
        self.typed(|c| c.as_view().cloned())
    }

    pub fn native_views(&self) -> Vec<(PluginId, NativeViewSpec)> {
        self.typed(|c| c.as_native_view().cloned())
    }

    pub fn audio_processors(&self) -> Vec<(PluginId, AudioProcessorSpec)> {
        let mut items = self.typed(|c| c.as_audio_processor().cloned());
        items.sort_by_key(|(_, spec)| spec.order);
        items
    }

    pub fn settings(&self, plugin: &PluginId) -> Vec<SettingSpec> {
        self.typed(|c| c.as_setting().cloned())
            .into_iter()
            .filter(|(owner, _)| owner == plugin)
            .map(|(_, spec)| spec)
            .collect()
    }

    pub fn owner_of_command(&self, command: &CommandId) -> Option<PluginId> {
        self.commands()
            .into_iter()
            .find(|(_, spec)| &spec.id == command)
            .map(|(plugin, _)| plugin)
    }

    fn typed<T>(&self, extract: impl Fn(&Contribution) -> Option<T>) -> Vec<(PluginId, T)> {
        let Ok(lock) = self.buckets.read() else {
            return Vec::new();
        };
        lock.values()
            .flatten()
            .filter_map(|record| {
                extract(&record.contribution).map(|value| (record.plugin.clone(), value))
            })
            .collect()
    }

    fn typed_filter<T>(&self, extract: impl Fn(&Contribution) -> Option<T>) -> Vec<(PluginId, T)> {
        self.typed(extract)
    }

    pub fn snapshot(&self) -> Vec<ContributionSnapshotItem> {
        let Ok(lock) = self.buckets.read() else {
            return Vec::new();
        };
        let mut items: Vec<ContributionSnapshotItem> = lock
            .iter()
            .flat_map(|(point, bucket)| {
                bucket.iter().map(move |record| ContributionSnapshotItem {
                    point: point.clone(),
                    plugin: record.plugin.clone(),
                    key: record.contribution.key(),
                })
            })
            .collect();
        items.sort_by(|a, b| (&a.point, &a.plugin, &a.key).cmp(&(&b.point, &b.plugin, &b.key)));
        items
    }

    pub fn rich_snapshot(&self) -> Vec<RichContribution> {
        let Ok(lock) = self.buckets.read() else {
            return Vec::new();
        };
        let mut items: Vec<RichContribution> = lock
            .iter()
            .flat_map(|(point, bucket)| {
                bucket.iter().map(move |record| RichContribution {
                    point: point.clone(),
                    plugin: record.plugin.clone(),
                    key: record.contribution.key(),
                    payload: serde_json::to_value(&record.contribution)
                        .unwrap_or(serde_json::Value::Null),
                })
            })
            .collect();
        items.sort_by(|a, b| {
            (&a.point, &a.plugin, &a.key).cmp(&(&b.point, &b.plugin, &b.key))
        });
        items
    }
}

impl Default for ContributionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionSnapshotItem {
    pub point: ContributionPointId,
    pub plugin: PluginId,
    pub key: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichContribution {
    pub point: ContributionPointId,
    pub plugin: PluginId,
    pub key: String,
    pub payload: serde_json::Value,
}
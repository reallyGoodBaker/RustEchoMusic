use std::path::{Path, PathBuf};

use plugin_sdk::{
    validate_manifest, DiscoveredPlugin, Manifest, PluginSource, Version,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryIssue {
    pub path: PathBuf,
    pub plugin: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct DiscoveryReport {
    pub found: Vec<DiscoveredPlugin>,
    pub issues: Vec<DiscoveryIssue>,
}

impl DiscoveryReport {
    pub fn merge(&mut self, other: DiscoveryReport) {
        self.found.extend(other.found);
        self.issues.extend(other.issues);
    }

    pub fn ids(&self) -> Vec<String> {
        self.found
            .iter()
            .map(|item| item.manifest.id.to_string())
            .collect()
    }
}

pub trait PluginLocator: Send + Sync {
    fn name(&self) -> &'static str;
    fn locate(&self, host_version: &Version) -> DiscoveryReport;
}

pub struct BuiltinLocator {
    entries: Vec<(Manifest, PathBuf)>,
}

impl BuiltinLocator {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register(mut self, manifest: Manifest, root: impl Into<PathBuf>) -> Self {
        self.entries.push((manifest, root.into()));
        self
    }
}

impl Default for BuiltinLocator {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginLocator for BuiltinLocator {
    fn name(&self) -> &'static str {
        "builtin"
    }

    fn locate(&self, host_version: &Version) -> DiscoveryReport {
        let mut report = DiscoveryReport::default();
        for (manifest, root) in &self.entries {
            match validate_manifest(manifest, host_version) {
                Ok(()) => report.found.push(DiscoveredPlugin {
                    manifest: manifest.clone(),
                    root: root.clone(),
                }),
                Err(error) => report.issues.push(DiscoveryIssue {
                    path: root.clone(),
                    plugin: Some(manifest.id.to_string()),
                    message: error.to_string(),
                }),
            }
        }
        report
    }
}

pub struct DirectoryLocator {
    root: PathBuf,
    source: PluginSource,
}

impl DirectoryLocator {
    pub fn new(root: impl Into<PathBuf>, source: PluginSource) -> Self {
        Self {
            root: root.into(),
            source,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn inspect(&self, dir: &Path, host_version: &Version) -> Result<DiscoveredPlugin, DiscoveryIssue> {
        let manifest_path = dir.join("plugin.json");
        let fail = |message: String| DiscoveryIssue {
            path: dir.to_path_buf(),
            plugin: None,
            message,
        };

        if !manifest_path.exists() {
            return Err(fail("missing plugin.json".into()));
        }
        let content =
            std::fs::read_to_string(&manifest_path).map_err(|e| fail(format!("unreadable: {e}")))?;
        let mut manifest: Manifest =
            serde_json::from_str(&content).map_err(|e| fail(format!("invalid plugin.json: {e}")))?;

        // 强制改写成定位器自身的来源：目录里的 json 说了不算。
        manifest.source = self.source;
        validate_manifest(&manifest, host_version)
            .map_err(|e| fail(e.to_string()))
            .map(|()| DiscoveredPlugin {
                manifest,
                root: dir.to_path_buf(),
            })
    }
}

impl PluginLocator for DirectoryLocator {
    fn name(&self) -> &'static str {
        match self.source {
            PluginSource::Builtin => "builtin-directory",
            PluginSource::Packaged => "packaged-directory",
            PluginSource::User => "user-directory",
        }
    }

    fn locate(&self, host_version: &Version) -> DiscoveryReport {
        let mut report = DiscoveryReport::default();
        if !self.root.exists() {
            // 目录不存在是正常情况（首次启动），不是错误。
            return report;
        }
        let entries = match std::fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(error) => {
                report.issues.push(DiscoveryIssue {
                    path: self.root.clone(),
                    plugin: None,
                    message: format!("cannot read plugin directory: {error}"),
                });
                return report;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            match self.inspect(&path, host_version) {
                Ok(plugin) => report.found.push(plugin),
                Err(issue) => report.issues.push(issue),
            }
        }
        report
    }
}

pub struct CompositeLocator {
    inner: Vec<Box<dyn PluginLocator>>,
}

impl CompositeLocator {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn add(mut self, locator: impl PluginLocator + 'static) -> Self {
        self.inner.push(Box::new(locator));
        self
    }

    pub fn locate_all(&self, host_version: &Version) -> DiscoveryReport {
        let mut report = DiscoveryReport::default();
        for locator in &self.inner {
            let partial = locator.locate(host_version);
            report.merge(partial);
        }
        self.dedupe(&mut report);
        report
    }

    fn dedupe(&self, report: &mut DiscoveryReport) {
        let mut seen: Vec<String> = Vec::new();
        let mut kept = Vec::with_capacity(report.found.len());
        for item in report.found.drain(..) {
            let id = item.manifest.id.to_string();
            if seen.contains(&id) {
                report.issues.push(DiscoveryIssue {
                    path: item.root.clone(),
                    plugin: Some(id.clone()),
                    message: format!("duplicate plugin id '{id}'; the earlier one wins"),
                });
                continue;
            }
            seen.push(id);
            kept.push(item);
        }
        report.found = kept;
    }
}

impl Default for CompositeLocator {
    fn default() -> Self {
        Self::new()
    }
}
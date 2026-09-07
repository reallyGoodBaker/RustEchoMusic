use std::collections::{HashMap, HashSet, VecDeque};

use plugin_sdk::{DependencySpec, DiscoveredPlugin, PluginError, PluginId, PluginResult, Version};

#[derive(Debug, Clone, Default)]
pub struct DependencyGraph {
    edges: HashMap<PluginId, HashSet<PluginId>>,
    dependents: HashMap<PluginId, HashSet<PluginId>>,
    min_versions: HashMap<(PluginId, PluginId), Version>,
    versions: HashMap<PluginId, Version>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build<'a>(plugins: impl IntoIterator<Item = &'a DiscoveredPlugin>) -> Self {
        let mut graph = Self::new();
        let items: Vec<&DiscoveredPlugin> = plugins.into_iter().collect();

        for item in &items {
            graph
                .versions
                .insert(item.manifest.id.clone(), item.manifest.version.clone());
            graph.edges.entry(item.manifest.id.clone()).or_default();
            graph.dependents.entry(item.manifest.id.clone()).or_default();
        }

        for item in &items {
            for dependency in &item.manifest.dependencies {
                // 反向边无论强弱都记录：软依赖的提供方被卸载时，
                // 依赖方虽能继续运行，但应收到一次"能力已消失"的事件。
                graph
                    .dependents
                    .entry(dependency.plugin.clone())
                    .or_default()
                    .insert(item.manifest.id.clone());

                if !dependency.optional {
                    graph
                        .edges
                        .entry(item.manifest.id.clone())
                        .or_default()
                        .insert(dependency.plugin.clone());
                    if let Some(min) = &dependency.min_version {
                        graph.min_versions.insert(
                            (item.manifest.id.clone(), dependency.plugin.clone()),
                            min.clone(),
                        );
                    }
                }
            }
        }
        graph
    }

    pub fn missing_dependencies(&self, plugin: &PluginId) -> Vec<PluginId> {
        let available = self.versions.keys().collect::<HashSet<_>>();
        self.edges
            .get(plugin)
            .map(|deps| {
                deps.iter()
                    .filter(|dep| !available.contains(dep))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn version_conflicts(&self, plugin: &PluginId) -> Vec<(PluginId, Version, Version)> {
        let mut conflicts = Vec::new();
        if let Some(deps) = self.edges.get(plugin) {
            for dep in deps {
                if let Some(required) = self.min_versions.get(&(plugin.clone(), dep.clone())) {
                    let actual = self.versions.get(dep);
                    match actual {
                        None => {}
                        Some(actual) if actual < required => {
                            conflicts.push((dep.clone(), required.clone(), actual.clone()))
                        }
                        Some(_) => {}
                    }
                }
            }
        }
        conflicts.sort();
        conflicts
    }

    pub fn dependents_of(&self, plugin: &PluginId) -> Vec<PluginId> {
        let mut dependents: Vec<PluginId> = self
            .dependents
            .get(plugin)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default();
        dependents.sort();
        dependents
    }

    pub fn activation_order(
        &self,
        targets: &[PluginId],
    ) -> PluginResult<Vec<PluginId>> {
        let targets: HashSet<&PluginId> = targets.iter().collect();

        // 前置校验：缺依赖 / 版本不符。
        for target in &targets {
            let missing = self.missing_dependencies(target);
            if let Some(first) = missing.first() {
                return Err(PluginError::dependency_missing(format!(
                    "plugin '{target}' depends on '{first}', which is not installed"
                )));
            }
            let conflicts = self.version_conflicts(target);
            if let Some((dep, required, actual)) = conflicts.first() {
                return Err(PluginError::dependency_missing(format!(
                    "plugin '{target}' needs '{dep}' >= {required}, but {actual} is installed"
                )));
            }
        }

        // 闭包：目标集 = targets ∪ 其全部传递强依赖。
        let mut closure: HashSet<PluginId> = HashSet::new();
        let mut queue: VecDeque<PluginId> = targets.iter().cloned().cloned().collect();
        while let Some(current) = queue.pop_front() {
            if !closure.insert(current.clone()) {
                continue;
            }
            if let Some(deps) = self.edges.get(&current) {
                for dep in deps {
                    queue.push_back(dep.clone());
                }
            }
        }

        // Kahn 算法，入度 = 该插件在闭包内依赖的、尚未安排的插件数。
        let mut indegree: HashMap<PluginId, usize> = closure
            .iter()
            .map(|id| {
                let count = self
                    .edges
                    .get(id)
                    .map(|deps| deps.iter().filter(|d| closure.contains(*d)).count())
                    .unwrap_or(0);
                (id.clone(), count)
            })
            .collect();

        let mut ready: Vec<PluginId> = indegree
            .iter()
            .filter(|(_, &count)| count == 0)
            .map(|(id, _)| id.clone())
            .collect();
        // 排序保证输出稳定，避免 HashMap 迭代顺序导致的随机激活顺序。
        ready.sort();

        let mut ordered = Vec::with_capacity(closure.len());
        while let Some(current) = ready.first().cloned() {
            ready.remove(0);
            ordered.push(current.clone());
            for dependent in self.dependents.get(&current).into_iter().flatten() {
                if !closure.contains(dependent) {
                    continue;
                }
                if let Some(count) = indegree.get_mut(dependent) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        ready.push(dependent.clone());
                        ready.sort();
                    }
                }
            }
        }

        if ordered.len() != closure.len() {
            let ordered_set: HashSet<PluginId> = ordered.iter().cloned().collect();
            let remaining: Vec<String> = closure
                .difference(&ordered_set)
                .map(|id| id.to_string())
                .collect();
            return Err(PluginError::invalid_argument(format!(
                "dependency cycle detected among: {}",
                remaining.join(", ")
            )));
        }
        Ok(ordered)
    }

    pub fn deactivation_order(&self, targets: &[PluginId]) -> Vec<PluginId> {
        let mut order = self.activation_order(targets).unwrap_or_else(|_| {
            // 顺序不可解时就按原顺序停用，至少不会漏。
            targets.to_vec()
        });
        order.reverse();
        order
    }

    pub fn unload_impact(&self, plugin: &PluginId) -> Vec<PluginId> {
        let mut affected: Vec<PluginId> = vec![plugin.clone()];
        let mut queue: VecDeque<PluginId> = VecDeque::from(self.dependents_of(plugin));
        let mut seen: HashSet<PluginId> = std::iter::once(plugin.clone()).collect();
        while let Some(current) = queue.pop_front() {
            if !seen.insert(current.clone()) {
                continue;
            }
            affected.push(current.clone());
            for dependent in self.dependents_of(&current) {
                queue.push_back(dependent);
            }
        }
        // 依赖者在前、被依赖者在后。
        affected.reverse();
        affected
    }
}

pub fn declared_providers(manifest: &plugin_sdk::Manifest) -> Vec<PluginId> {
    manifest
        .dependencies
        .iter()
        .map(|d: &DependencySpec| d.plugin.clone())
        .collect()
}
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Trait that all plugins must implement
#[allow(dead_code)]
use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn help(&self) -> &str;
    fn execute(&self, args: &[String]) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub metadata: PluginMetadata,
    pub help: String,
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    libs: Vec<Library>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: vec![], libs: vec![] }
    }

    pub fn load_plugin(&mut self, path: &Path) -> Result<()> {
        unsafe {
            let lib = Library::new(path)
                .with_context(|| format!("Failed to load plugin: {:?}", path))?;

            let plugin_init: Symbol<unsafe extern fn() -> *mut dyn Plugin> = lib
                .get(b"_plugin_init")
                .with_context(|| format!("Failed to find _plugin_init in {:?}", path))?;

            let plugin = Box::from_raw(plugin_init());
            self.plugins.push(plugin);
            self.libs.push(lib);
        }
        Ok(())
    }

    pub fn load_plugins_from_dir(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "so" || ext == "dylib" || ext == "dll" {
                        self.load_plugin(&path)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins
            .iter()
            .map(|p| PluginMetadata {
                name: p.name().to_string(),
                description: p.description().to_string(),
                version: p.version().to_string(),
                author: p.author().to_string(),
                path: PathBuf::new(), // This is not ideal, but we don't have the path here
            })
            .collect()
    }

    pub fn execute_plugin(&self, name: &str, args: &[String]) -> Result<()> {
        if let Some(plugin) = self.plugins.iter().find(|p| p.name() == name) {
            plugin.execute(args)
        } else {
            anyhow::bail!("Plugin not found: {}", name)
        }
    }

    pub fn remove_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(index) = self.plugins.iter().position(|p| p.name() == name) {
            self.plugins.remove(index);
            self.libs.remove(index);
        }
        Ok(())
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.name().to_string()).collect()
    }

    pub fn get_plugin_info(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.iter().find(|p| p.name() == name).map(|p| {
            let metadata = PluginMetadata {
                name: p.name().to_string(),
                description: p.description().to_string(),
                version: p.version().to_string(),
                author: p.author().to_string(),
                path: PathBuf::new(),
            };
            PluginInfo {
                metadata,
                help: p.help().to_string(),
            }
        })
    }
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:expr) => {
        #[no_mangle]
        pub extern "C" fn _plugin_init() -> *mut dyn $crate::plugin::Plugin {
            let constructor: fn() -> $plugin_type = $constructor;
            let object = constructor();
            let boxed: Box<dyn $crate::plugin::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}

pub struct ExamplePlugin {
    name: String,
    description: String,
    version: String,
    author: String,
}

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn author(&self) -> &str {
        &self.author
    }

    fn help(&self) -> &str {
        "This is an example plugin."
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        println!("Hello from the example plugin!");
        Ok(())
    }
}

pub fn init_plugin_system() -> Result<PluginManager> {
    let mut manager = PluginManager::new();

    // Load plugins from a known directory
    if let Some(home_dir) = dirs::home_dir() {
        let plugin_dir = home_dir.join(".saorsa-cli/plugins");
        manager.load_plugins_from_dir(&plugin_dir)?;
    }

    Ok(manager)
}


/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub path: PathBuf,
}

/// Detailed plugin information
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub help: String,
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    plugin_dirs: Vec<PathBuf>,
    plugin_metadata: HashMap<String, PluginMetadata>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dirs: Vec::new(),
            plugin_metadata: HashMap::new(),
        }
    }

    /// Add a plugin directory to search for plugins
    pub fn add_plugin_dir(&mut self, dir: PathBuf) {
        self.plugin_dirs.push(dir);
    }

    /// Load all plugins from configured directories
    pub fn load_plugins(&mut self) -> Result<()> {
        let dirs = self.plugin_dirs.clone();
        for dir in dirs {
            self.load_plugins_from_dir(&dir)?;
        }
        Ok(())
    }

    /// Load plugins from a specific directory
    fn load_plugins_from_dir(&mut self, dir: &PathBuf) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        // For now, we'll implement a simple plugin system
        // In a real implementation, this would load dynamic libraries
        // and instantiate plugin objects

        // Example: Load built-in plugins
        self.load_builtin_plugins();

        Ok(())
    }

    /// Load built-in example plugins
    fn load_builtin_plugins(&mut self) {
        let example_plugin = ExamplePlugin::new();
        let name = example_plugin.name().to_string();
        let metadata = PluginMetadata {
            name: name.clone(),
            description: example_plugin.description().to_string(),
            version: example_plugin.version().to_string(),
            path: PathBuf::from("builtin"),
        };

        self.plugins.insert(name.clone(), Box::new(example_plugin));
        self.plugin_metadata.insert(name, metadata);
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugin_metadata.values().cloned().collect()
    }

    /// Execute a plugin by name
    pub fn execute_plugin(&self, name: &str, args: &[String]) -> Result<()> {
        if let Some(plugin) = self.get_plugin(name) {
            plugin.execute(args)?;
        } else {
            anyhow::bail!("Plugin '{}' not found", name);
        }
        Ok(())
    }

    /// Get plugin directories
    pub fn plugin_dirs(&self) -> &[PathBuf] {
        &self.plugin_dirs
    }

    /// Remove a plugin by name
    pub fn remove_plugin(&mut self, name: &str) -> Result<()> {
        if self.plugins.remove(name).is_some() {
            println!("✓ Plugin '{}' removed successfully", name);
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name);
        }
    }

    /// Get plugin count
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    /// Get plugin names for selection
    pub fn get_plugin_names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Get detailed information about a plugin
    pub fn get_plugin_info(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.get(name).map(|plugin| PluginInfo {
            name: plugin.name().to_string(),
            description: plugin.description().to_string(),
            version: plugin.version().to_string(),
            help: plugin.help().to_string(),
        })
    }
}

/// Example plugin implementation
pub struct ExamplePlugin {
    name: String,
    description: String,
    version: String,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            name: "example".to_string(),
            description: "An example plugin demonstrating the plugin system".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn help(&self) -> &str {
        "Example plugin that demonstrates the plugin system"
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        println!("Example plugin executed with args: {:?}", args);
        println!("Hello from the example plugin!");
        Ok(())
    }
}

/// Plugin system initialization
pub fn init_plugin_system() -> Result<PluginManager> {
    let mut manager = PluginManager::new();

    // Add default plugin directories
    if let Some(home_dir) = dirs::home_dir() {
        let plugin_dir = home_dir.join(".saorsa-cli").join("plugins");
        manager.add_plugin_dir(plugin_dir);
    }

    // Add system plugin directory
    manager.add_plugin_dir(PathBuf::from("/usr/local/lib/saorsa-cli/plugins"));

    // Load plugins
    manager.load_plugins()?;

    Ok(manager)
}

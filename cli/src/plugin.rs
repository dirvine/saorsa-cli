use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get the plugin name
    fn name(&self) -> &str;

    /// Get the plugin description
    fn description(&self) -> &str;

    /// Get the plugin version
    fn version(&self) -> &str;

    /// Get plugin help text
    fn help(&self) -> &str;

    /// Execute the plugin with given arguments
    fn execute(&self, args: &[String]) -> Result<()>;
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

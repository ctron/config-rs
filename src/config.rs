use std::fmt::Debug;

use crate::builder::{ConfigBuilder, DefaultState};
use serde::de::Deserialize;
use serde::ser::Serialize;

use crate::error::*;
use crate::map::Map;
use crate::path;
use crate::ser::ConfigSerializer;
use crate::source::Source;
use crate::value::{Table, Value};

/// A prioritized configuration repository. It maintains a set of
/// configuration sources, fetches values to populate those, and provides
/// them according to the source's priority.
#[derive(Clone, Debug)]
pub struct Config {
    defaults: Map<path::Expression, Value>,
    overrides: Map<path::Expression, Value>,
    sources: Vec<Box<dyn Source + Send + Sync>>,

    /// Root of the cached configuration.
    pub cache: Value,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            defaults: Default::default(),
            overrides: Default::default(),
            sources: Default::default(),
            cache: Value::new(None, Table::new()),
        }
    }
}

impl Config {
    pub(crate) fn new(value: Value) -> Self {
        Config {
            cache: value,
            ..Default::default()
        }
    }

    /// Creates new [`ConfigBuilder`] instance
    pub fn builder() -> ConfigBuilder<DefaultState> {
        ConfigBuilder::<DefaultState>::default()
    }

    /// Merge in a configuration property source.
    #[deprecated(since = "0.12.0", note = "please use 'ConfigBuilder' instead")]
    pub fn merge<T>(&mut self, source: T) -> Result<&mut Config>
    where
        T: 'static,
        T: Source + Send + Sync,
    {
        self.sources.push(Box::new(source));

        #[allow(deprecated)]
        self.refresh()
    }

    /// Merge in a configuration property source.
    #[deprecated(since = "0.12.0", note = "please use 'ConfigBuilder' instead")]
    pub fn with_merged<T>(mut self, source: T) -> Result<Self>
    where
        T: 'static,
        T: Source + Send + Sync,
    {
        self.sources.push(Box::new(source));

        #[allow(deprecated)]
        self.refresh()?;
        Ok(self)
    }

    /// Refresh the configuration cache with fresh
    /// data from added sources.
    ///
    /// Configuration is automatically refreshed after a mutation
    /// operation (`set`, `merge`, `set_default`, etc.).
    #[deprecated(since = "0.12.0", note = "please use 'ConfigBuilder' instead")]
    pub fn refresh(&mut self) -> Result<&mut Config> {
        self.cache = {
            let mut cache: Value = Map::<String, Value>::new().into();

            // Add defaults
            for (key, val) in self.defaults.iter() {
                key.set(&mut cache, val.clone());
            }

            // Add sources
            self.sources.collect_to(&mut cache)?;

            // Add overrides
            for (key, val) in self.overrides.iter() {
                key.set(&mut cache, val.clone());
            }

            cache
        };

        Ok(self)
    }

    /// Set a default `value` at `key`
    #[deprecated(since = "0.12.0", note = "please use 'ConfigBuilder' instead")]
    pub fn set_default<T>(&mut self, key: &str, value: T) -> Result<&mut Config>
    where
        T: Into<Value>,
    {
        self.defaults.insert(key.parse()?, value.into());

        #[allow(deprecated)]
        self.refresh()
    }

    /// Set an overwrite
    ///
    /// This function sets an overwrite value.
    /// The overwrite `value` is written to the `key` location on every `refresh()`
    ///
    /// # Warning
    ///
    /// Errors if config is frozen
    #[deprecated(since = "0.12.0", note = "please use 'ConfigBuilder' instead")]
    pub fn set<T>(&mut self, key: &str, value: T) -> Result<&mut Config>
    where
        T: Into<Value>,
    {
        self.overrides.insert(key.parse()?, value.into());

        #[allow(deprecated)]
        self.refresh()
    }

    #[deprecated(since = "0.12.0", note = "please use 'ConfigBuilder' instead")]
    pub fn set_once(&mut self, key: &str, value: Value) -> Result<()> {
        let expr: path::Expression = key.parse()?;

        // Traverse the cache using the path to (possibly) retrieve a value
        if let Some(ref mut val) = expr.get_mut(&mut self.cache) {
            **val = value;
        } else {
            expr.set(&mut self.cache, value);
        }
        Ok(())
    }

    pub fn get<'de, T: Deserialize<'de>>(&self, key: &str) -> Result<T> {
        // Parse the key into a path expression
        let expr: path::Expression = key.parse()?;

        // Traverse the cache using the path to (possibly) retrieve a value
        let value = expr.get(&self.cache).cloned();

        match value {
            Some(value) => {
                // Deserialize the received value into the requested type
                T::deserialize(value).map_err(|e| e.extend_with_key(key))
            }

            None => Err(ConfigError::NotFound(key.into())),
        }
    }

    pub fn get_string(&self, key: &str) -> Result<String> {
        self.get(key).and_then(Value::into_string)
    }

    pub fn get_int(&self, key: &str) -> Result<i64> {
        self.get(key).and_then(Value::into_int)
    }

    pub fn get_float(&self, key: &str) -> Result<f64> {
        self.get(key).and_then(Value::into_float)
    }

    pub fn get_bool(&self, key: &str) -> Result<bool> {
        self.get(key).and_then(Value::into_bool)
    }

    pub fn get_table(&self, key: &str) -> Result<Map<String, Value>> {
        self.get(key).and_then(Value::into_table)
    }

    pub fn get_array(&self, key: &str) -> Result<Vec<Value>> {
        self.get(key).and_then(Value::into_array)
    }

    /// Attempt to deserialize the entire configuration into the requested type.
    pub fn try_into<'de, T: Deserialize<'de>>(self) -> Result<T> {
        T::deserialize(self)
    }

    /// Attempt to serialize the entire configuration from the given type.
    pub fn try_from<T: Serialize>(from: &T) -> Result<Self> {
        let mut serializer = ConfigSerializer::default();
        from.serialize(&mut serializer)?;
        Ok(serializer.output)
    }

    #[deprecated(since = "0.7.0", note = "please use 'try_into' instead")]
    pub fn deserialize<'de, T: Deserialize<'de>>(self) -> Result<T> {
        self.try_into()
    }
}

impl Source for Config {
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<Map<String, Value>> {
        self.cache.clone().into_table()
    }
}

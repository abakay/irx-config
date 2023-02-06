//! This module define [`Value`] structure which represent key-value based configuration data.

use crate::{Error, Result, DEFAULT_KEYS_SEPARATOR};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use serde_json::json;
pub(super) use serde_json::Error as SerdeError;
use serde_json::{map::Map, Value as InnerValue};
use std::borrow::Cow;
use std::fmt::{Debug, Display, Error as FmtError, Formatter, Result as FmtResult};
use std::result::Result as StdResult;

type ValueMap = Map<String, InnerValue>;
type CowInnerValue<'a> = Cow<'a, InnerValue>;

/// The sealed states for [`Value`] structure.
///
/// If [`Value`] is sealed, the sensitive fields values will be obfuscated with `********` during display/debugging output.
///
/// **IMPORTANT:** Once [`Value`] was sealed, but fully/partially mutated after that, it will be represented as empty
/// dictionary during display/debugging output, to prevent sensitive data leakages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SealedState {
    /// A [`Value`] was never sealed, all data will be represented as is during display/debugging output.
    #[default]
    None,
    /// A [`Value`] was sealed, sensitive data will be obfuscated during display/debugging output.
    On,
    /// A [`Value`] was sealed and fully/partially mutated after that, whole [`Value`] will be represented as empty
    /// dictionary during display/debugging output (see above).
    Mutated,
}

/// This structure represent key-value based configuration data.
///
/// **IMPORTANT:** All functionality related to the sealed state only affects the display/debugging output.
#[derive(Clone)]
pub struct Value {
    value: InnerValue,
    sealed: Option<InnerValue>,
    sealed_state: SealedState,
    case_on: bool,
}

impl Value {
    /// Try to create [`Value`] structure from any type which implements [`Serialize`] trait and make key names to be
    /// case-sensitive. If successful return instance of [`Value`] structure.
    ///
    /// # Errors
    ///
    /// If any errors will occur during construction then error will be returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::{json, Value};
    ///
    /// let data = Value::try_from(json!(
    /// {
    ///     "name": "John Doe",
    ///     "age": 43,
    ///     "phones": [
    ///         "+44 1234567",
    ///         "+44 2345678"
    ///     ]
    /// }))?;
    /// ```
    #[inline]
    pub fn try_from<T: Serialize>(value: T) -> Result<Self> {
        Self::try_from_with_case(value, true)
    }

    /// Try to create [`Value`] structure from any type which implements [`Serialize`] trait and make key names to be
    /// case-sensitive/insensitive. If successful return instance of [`Value`] structure.
    ///
    /// # Errors
    ///
    /// If any errors will occur during construction then error will be returned.
    #[inline]
    pub fn try_from_with_case<T: Serialize>(value: T, case_on: bool) -> Result<Self> {
        Ok(Self {
            value: set(value, case_on)?,
            case_on,
            ..Default::default()
        })
    }

    /// Create default [`Value`] structure with key names to be case-sensitive/insensitive.
    #[inline]
    pub fn with_case(on: bool) -> Self {
        Self {
            case_on: on,
            ..Default::default()
        }
    }

    /// Return `true` if key names is case-sensitive, otherwise return `false`.
    #[inline]
    pub fn is_case_sensitive(&self) -> bool {
        self.case_on
    }

    /// Merge a input [`Value`] to the given [`Value`] structure. The key names will use case-sensitivity of given
    /// [`Value`] during merge. Return merged result [`Value`] structure. If given [`Value`] was sealed and merge
    /// operation was mutating then it will be in [`SealedState::Mutated`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::{json, Value};
    ///
    /// let mut person = Value::try_from(json!({
    ///     "name": "John Doe",
    ///     "age": 43
    /// }))?;
    ///
    /// let phones = Value::try_from(json!({
    ///     "phones": [
    ///         "+44 1234567",
    ///         "+44 2345678"
    ///     ]
    /// }))?;
    ///
    /// person = person.merge(&phones);
    /// ```
    #[inline]
    pub fn merge(self, value: &Value) -> Self {
        let case_on = self.case_on;
        self.merge_with_case(value, case_on)
    }

    /// Merge a input [`Value`] to the given [`Value`] structure. The key names will be case-sensitive or
    /// case-insensitive during merge, according to `case_on` parameter. Return merged result [`Value`] structure.
    /// If given [`Value`] was sealed and merge operation was mutating then it will be in [`SealedState::Mutated`].
    pub fn merge_with_case(mut self, value: &Value, case_on: bool) -> Self {
        let mut is_changed = self.normalize_case(case_on);
        self.value = match self.value {
            InnerValue::Object(dst) if value.value.is_object() => {
                is_changed = true;
                merge_into_value_map(dst, &value.value, self.case_on)
            }
            _ => self.value,
        };

        if is_changed {
            self.unseal();
        }
        self
    }

    /// Return deserialized data of any type which implements [`Deserialize`] trait for given key path represented
    /// as iterator. If given key path does not exists `Ok(None)` will be returned.
    ///
    /// # Errors
    ///
    /// If any errors will occur then error will be returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::{json, Value};
    ///
    /// let logger = Value::try_from(json!({
    ///     "logger": {
    ///         "id": 42,
    ///         "host": "localhost"
    ///     }
    /// }))?;
    ///
    /// let id: u32 = logger.get_by_keys(["logger", "id"])?.unwrap();
    /// ```
    pub fn get_by_keys<I, K, T>(&self, keys: I) -> Result<Option<T>>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
        T: DeserializeOwned,
    {
        let mut result = &self.value;
        for key in keys {
            result = if let InnerValue::Object(map) = result {
                let key = crate::normalize_case(key.as_ref(), self.case_on);
                match map.get(key.as_ref()) {
                    None => return Ok(None),
                    Some(v) => v,
                }
            } else {
                return Ok(None);
            };
        }

        Ok(Some(get(result.clone())?))
    }

    /// Return deserialized data of any type which implements [`Deserialize`] trait for given key path represented
    /// as string with default keys level delimiter [`DEFAULT_KEYS_SEPARATOR`]. If given key path does not exists
    /// `Ok(None)` will be returned.
    ///
    /// # Errors
    ///
    /// If any errors will occur then error will be returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::{json, Value};
    ///
    /// let logger = Value::try_from(json!({
    ///     "logger": {
    ///         "id": 42,
    ///         "host": "localhost"
    ///     }
    /// }))?;
    ///
    /// let id: u32 = logger.get_by_key_path("logger:id")?.unwrap();
    /// ```
    #[inline]
    pub fn get_by_key_path<T, P>(&self, path: P) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        P: AsRef<str>,
    {
        self.get_by_key_path_with_delim(path.as_ref(), DEFAULT_KEYS_SEPARATOR)
    }

    /// Return deserialized data of any type which implements [`Deserialize`] trait for given key path represented
    /// as string with given keys level delimiter. If given key path does not exists `Ok(None)` will be returned.
    ///
    /// # Errors
    ///
    /// If any errors will occur then error will be returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::{json, Value};
    ///
    /// let logger = Value::try_from(json!({
    ///     "logger": {
    ///         "id": 42,
    ///         "host": "localhost"
    ///     }
    /// }))?;
    ///
    /// let host: String = logger.get_by_key_path_with_delim("logger/host", "/")?.unwrap();
    /// ```
    pub fn get_by_key_path_with_delim<T, P, D>(&self, path: P, delim: D) -> Result<Option<T>>
    where
        T: DeserializeOwned,
        P: AsRef<str>,
        D: AsRef<str>,
    {
        fn inner<T>(value: &Value, path: &str, delim: &str) -> Result<Option<T>>
        where
            T: DeserializeOwned,
        {
            if delim.is_empty() {
                return Err(Error::EmptySeparator);
            }

            if path.is_empty() {
                return value.get_by_keys([""; 0]);
            }

            value.get_by_keys(path.split(delim))
        }

        inner(self, path.as_ref(), delim.as_ref())
    }

    /// Return deserialized data of any type which implements [`Deserialize`] trait.
    ///
    /// # Errors
    ///
    /// If any errors will occur then error will be returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::{json, Value};
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Logger {
    ///     pub id: u32,
    ///     pub host: String,
    /// }
    ///
    /// #[derive(Deserialize)]
    /// struct Config {
    ///     logger: Logger,
    /// }
    ///
    /// let config = Value::try_from(json!({
    ///     "logger": {
    ///         "id": 42,
    ///         "host": "localhost"
    ///     }
    /// }))?;
    ///
    /// let config: Config = config.get()?;
    /// ```
    #[inline]
    pub fn get<T: DeserializeOwned>(&self) -> Result<T> {
        get(self.value.clone())
    }

    /// Set value of any type which implements [`Serialize`] trait for given key path represented as iterator.
    /// If [`Value`] was sealed and set operation was successful then it will be in [`SealedState::Mutated`]. Return
    /// previous value for same key path if any.
    ///
    /// # Errors
    ///
    /// If any errors will occur then error will be returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::Value;
    ///
    /// let mut value = Value::default();
    /// value.set_by_keys(["logger", "id"], 42)?;
    /// ```
    pub fn set_by_keys<I, K, T>(&mut self, keys: I, value: T) -> Result<Option<Self>>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
        T: Serialize,
    {
        let inner = || {
            let mut result = &mut self.value;
            let mut keys = keys.into_iter().peekable();
            while let Some(key) = keys.next() {
                let key = crate::normalize_case(key.as_ref(), self.case_on);
                result = match result {
                    InnerValue::Object(m) if keys.peek().is_none() => {
                        return Ok(m
                            .insert(key.into_owned(), set(value, self.case_on)?)
                            .map(|r| Self {
                                value: r,
                                case_on: self.case_on,
                                ..Default::default()
                            }))
                    }
                    InnerValue::Object(m) => m.entry(key).or_insert_with(|| json!({})),
                    _ => return Err(Error::NotMap),
                }
            }

            let prev = self.clone();
            self.value = set(value, self.case_on)?;
            Ok(Some(prev))
        };

        let result = inner()?;
        self.unseal();
        Ok(result)
    }

    /// Set value of any type which implements [`Serialize`] trait for given key path represented as string with default
    /// keys level delimiter [`DEFAULT_KEYS_SEPARATOR`]. If [`Value`] was sealed and set operation was successful then
    /// it will be in [`SealedState::Mutated`]. Return previous value for same key path if any.
    ///
    /// # Errors
    ///
    /// If any errors will occur then error will be returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::Value;
    ///
    /// let mut value = Value::default();
    /// value.set_by_key_path("logger:id", 42)?;
    /// ```
    #[inline]
    pub fn set_by_key_path<T, P>(&mut self, path: P, value: T) -> Result<Option<Self>>
    where
        T: Serialize,
        P: AsRef<str>,
    {
        self.set_by_key_path_with_delim(path.as_ref(), DEFAULT_KEYS_SEPARATOR, value)
    }

    /// Set value of any type which implements [`Serialize`] trait for given key path represented as string with given
    /// keys level delimiter. If [`Value`] was sealed and set operation was successful then it will be in
    /// [`SealedState::Mutated`]. Return previous value for same key path if any.
    ///
    /// # Errors
    ///
    /// If any errors will occur then error will be returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::Value;
    ///
    /// let mut value = Value::default();
    /// value.set_by_key_path_with_delim("logger/id", "/", 42)?;
    /// ```
    pub fn set_by_key_path_with_delim<T, P, D>(
        &mut self,
        path: P,
        delim: D,
        value: T,
    ) -> Result<Option<Self>>
    where
        T: Serialize,
        P: AsRef<str>,
        D: AsRef<str>,
    {
        fn inner<T>(this: &mut Value, path: &str, delim: &str, value: T) -> Result<Option<Value>>
        where
            T: Serialize,
        {
            if delim.is_empty() {
                return Err(Error::EmptySeparator);
            }

            if path.is_empty() {
                return this.set_by_keys([""; 0], value);
            }

            this.set_by_keys(path.split(delim), value)
        }

        inner(self, path.as_ref(), delim.as_ref(), value)
    }

    /// Return [`Value`] structure as a sequence of bytes.
    #[inline]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.value.to_string().as_bytes().to_owned()
    }

    /// Seal secret values in [`Value`] structure with given suffix. Such values will be obfuscated with `********`
    /// during display/debugging output. If not set then all values will be displayed as is.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irx_config::Value;
    ///
    /// let mut value = Value::try_from(json!({
    ///     "user": "user name",
    ///     "password_sealed_": "secret"
    /// }))?;
    ///
    /// value.seal("_sealed_");
    /// ```
    pub fn seal<S>(&mut self, suffix: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        fn inner<'a>(this: &'a mut Value, suffix: &str) -> &'a mut Value {
            if SealedState::On == this.sealed_state {
                return this;
            }

            this.sealed = None;
            this.sealed_state = SealedState::On;

            if suffix.is_empty() {
                return this;
            }

            if let InnerValue::Object(ref map) = this.value {
                let (v, s) = get_sealed(map, suffix, this.case_on);
                this.value = v;
                this.sealed = s;
            }
            this
        }

        inner(self, suffix.as_ref())
    }

    /// Return `true` if [`Value`] is sealed, otherwise return `false`.
    #[inline]
    pub fn is_sealed(&self) -> bool {
        SealedState::On == self.sealed_state
    }

    /// Return sealed state [`SealedState`].
    #[inline]
    pub fn sealed_state(&self) -> SealedState {
        self.sealed_state
    }

    fn normalize_case(&mut self, case_on: bool) -> bool {
        if case_on == self.case_on {
            return false;
        }

        if let InnerValue::Object(ref map) = self.value {
            self.case_on = case_on;
            if case_on {
                return false;
            }
            self.value = unicase_value_map(map);
            return true;
        }

        false
    }

    fn get_sealed(&self) -> CowInnerValue {
        if SealedState::Mutated == self.sealed_state {
            return CowInnerValue::Owned(json!({}));
        }

        if let Some(ref s) = self.sealed {
            if let InnerValue::Object(ref m) = self.value {
                return CowInnerValue::Owned(merge_into_value_map(m.clone(), s, self.case_on));
            }
        }
        CowInnerValue::Borrowed(&self.value)
    }

    fn unseal(&mut self) {
        if SealedState::On == self.sealed_state {
            self.sealed_state = SealedState::Mutated;
        }
        self.sealed = None;
    }
}

impl Default for Value {
    #[inline]
    fn default() -> Self {
        Self {
            value: json!({}),
            sealed: None,
            sealed_state: SealedState::None,
            case_on: true,
        }
    }
}

impl PartialEq for Value {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Value {}

impl Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self {
            value: InnerValue::deserialize(deserializer)?,
            ..Default::default()
        })
    }
}

impl Debug for Value {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!(
            "Value {{ value: {:?}, sealed: {:?}, sealed_state: {:?}, case_on: {:?} }}",
            self.get_sealed(),
            self.sealed,
            self.sealed_state,
            self.case_on
        ))
    }
}

impl Display for Value {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let value = serde_json::to_string_pretty(&self.get_sealed()).map_err(|_| FmtError)?;
        f.write_str(&value)
    }
}

fn merge_into_value_map(dst: ValueMap, src: &InnerValue, case_on: bool) -> InnerValue {
    if let InnerValue::Object(src) = src {
        let mut result = dst;
        for (k, v) in src {
            let norm_key = crate::normalize_case(k, case_on);
            let val = if let Some(InnerValue::Object(m)) = result.get(norm_key.as_ref()) {
                merge_into_value_map(m.clone(), v, case_on)
            } else {
                v.clone()
            };

            result.insert(norm_key.into_owned(), val);
        }
        return InnerValue::Object(result);
    }
    src.clone()
}

fn unicase_value_map(map: &ValueMap) -> InnerValue {
    let mut result = ValueMap::default();
    for (k, v) in map {
        let val = if let InnerValue::Object(m) = v {
            unicase_value_map(m)
        } else {
            v.clone()
        };
        result.insert(crate::unicase(k), val);
    }
    InnerValue::Object(result)
}

#[inline]
fn get<T: DeserializeOwned>(value: InnerValue) -> Result<T> {
    Ok(serde_json::from_value(value)?)
}

fn set<T: Serialize>(value: T, case_on: bool) -> Result<InnerValue> {
    let value = serde_json::to_value(value)?;
    Ok(match value {
        InnerValue::Object(ref map) if !case_on => unicase_value_map(map),
        _ => value,
    })
}

fn get_sealed(value: &ValueMap, suffix: &str, case_on: bool) -> (InnerValue, Option<InnerValue>) {
    let mut result = ValueMap::default();
    let mut sealed = ValueMap::default();
    let uni_suffix = crate::normalize_case(suffix, case_on);
    for (k, v) in value {
        let key = crate::normalize_case(k, case_on);
        let (val, opt) = if let Some(InnerValue::Object(nested)) = value.get(key.as_ref()) {
            let (v, s) = get_sealed(nested, suffix, case_on);
            (CowInnerValue::Owned(v), s)
        } else {
            (CowInnerValue::Borrowed(v), None)
        };

        let norm_key = key.trim_end_matches(uni_suffix.as_ref());
        result.insert(norm_key.to_string(), val.into_owned());
        if key.len() != norm_key.len() {
            sealed.insert(norm_key.to_string(), json!("********"));
        } else if let Some(s) = opt {
            sealed.insert(norm_key.to_string(), s);
        }
    }
    (
        InnerValue::Object(result),
        if sealed.is_empty() {
            None
        } else {
            Some(InnerValue::Object(sealed))
        },
    )
}

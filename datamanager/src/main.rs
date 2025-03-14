use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

#[derive(Clone)]
pub struct ParameterManager {
    params: HashMap<String, String>,
    param_rules: HashMap<String, ParamRule>,
    listeners: HashMap<String, Vec<Listener>>,
    wild_card_listeners: HashMap<String, Vec<Listener>>,
    listener_id_reverse: HashMap<usize, String>,
    listener_id: usize,
}

#[derive(Clone)]
pub struct Param {
    pub key: String,
    pub value: String,
}

#[derive(Clone)]
pub enum ParamType {
    TypeInt,
    TypeFloat,
    TypeBool,
    TypeString,
}

#[derive(Clone)]
pub enum ParamRange {
    RangeAny,
    Ranged,
    RangeEnum,
}

#[derive(Clone)]
pub struct ParamRule {
    pub param_type: ParamType,
    pub range: ParamRange,
    pub range_min: f32,
    pub range_max: f32,
    pub enum_vals: HashSet<String>,
}

#[derive(Clone)]
pub struct Listener {
    pub listener_id: usize,
    pub callback: Arc<Mutex<dyn Fn(String, String) + Send + Sync>>,
}

impl ParameterManager {
    pub fn new() -> Self {
        ParameterManager {
            params: HashMap::new(),
            param_rules: HashMap::new(),
            listeners: HashMap::new(),
            wild_card_listeners: HashMap::new(),
            listener_id_reverse: HashMap::new(),
            listener_id: 0,
        }
    }

    pub fn get_manager() -> Arc<Mutex<ParameterManager>> {
        static INSTANCE: Lazy<Arc<Mutex<Option<ParameterManager>>>> = Lazy::new(|| {
            Arc::new(Mutex::new(None))
        });

        let mut instance = INSTANCE.lock().unwrap();
        if instance.is_none() {
            *instance = Some(ParameterManager::new());
        }

        Arc::clone(&Arc::new(Mutex::new(instance.take().unwrap())))
    }

    pub fn set_parameter(&mut self, key: &str, value: &str) {
        let mut value = value.trim().to_string();
        if self.filter_value_with_rule(&key, &mut value) {
            let mut b_changed = true;

            if self.params.contains_key(key) {
                // Read-only key check (similar to "ro." check
                if key.starts_with("ro.") {
                    return;
                }

                b_changed = self.params[key] != value;
            }

            self.params.insert(key.to_string(), value.clone());

            if b_changed {
                for (a_key, listeners) in &self.wild_card_listeners {
                    if key.starts_with(a_key) {
                        self.execute_notify(&key, &value, listeners.clone());
                    }
                }

                if let Some(listeners) = self.listeners.get(key) {
                    self.execute_notify(&key, &value, listeners.clone());
                }
            }
        }
    }

    pub fn set_parameter_int(&mut self, key: &str, value: i32) {
        self.set_parameter(key, &value.to_string());
    }

    pub fn set_parameter_bool(&mut self, key: &str, value: bool) {
        self.set_parameter(key, &value.to_string());
    }


    pub fn register_callback<F>(&mut self, key: &str, callback: F) -> usize
    where
        F: Fn(String, String) + Send + Sync + 'static,
    {
        let listener_id = self.listener_id;
        self.listener_id += 1;

        let listener = Listener {
            listener_id,
            callback: Arc::new(Mutex::new(callback)),
        };

        self.listeners.entry(key.to_string()).or_insert_with(Vec::new).push(listener.clone());
        self.listener_id_reverse.insert(listener_id, key.to_string());

        listener_id
    }

    pub fn filter_value_with_rule(&self, key: &str, value: &mut String) -> bool {
        if let Some(rule) = self.param_rules.get(key) {
            match rule.range {
                ParamRange::RangeAny => {}
                ParamRange::Ranged => match rule.param_type {
                    ParamType::TypeInt => {
                        if let Ok(val) = value.parse::<i32>() {
                            let clamped_val = val.clamp(rule.range_min as i32, rule.range_max as i32);
                            *value = clamped_val.to_string();
                        }
                    }
                    ParamType::TypeFloat => {
                        if let Ok(val) = value.parse::<f32>() {
                            let clamped_val = val.clamp(rule.range_min, rule.range_max);
                            *value = clamped_val.to_string();
                        }
                    }
                    ParamType::TypeBool => {
                        if value == "true" || value == "false" {
                            *value = value.to_string();
                        } else {
                            return false;
                        }
                    }
                    ParamType::TypeString => {}
                },
                ParamRange::RangeEnum => {
                    if !rule.enum_vals.contains(value) {
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn execute_notify(&self, key: &str, value: &str, listeners: Vec<Listener>) {
        for listener in listeners {
            (listener.callback.lock().unwrap())(key.to_string(), value.to_string());
        }
    }

    pub fn get_parameter(&self, key: &str, default_value: &str) -> String {
        self.params.get(key).cloned().unwrap_or(default_value.to_string())
    }

    pub fn get_parameter_int(&self, key: &str, default_value: i32) -> i32 {
        self.params.get(key).and_then(|v| v.parse().ok()).unwrap_or(default_value)
    }

    pub fn get_parameter_float(&self, key: &str, default_value: f32) -> f32 {
        self.params.get(key).and_then(|v| v.parse().ok()).unwrap_or(default_value)
    }

    pub fn get_parameter_bool(&self, key: &str, default_value: bool) -> bool {
        self.params.get(key).map_or(default_value, |v| v == "true")
    }
}

fn main() {
    let param_manager = ParameterManager::get_manager();

    param_manager.lock().unwrap().set_parameter("example_key", "example_value");
    let value = param_manager.lock().unwrap().get_parameter("example_key", "default_value");
    println!("Parameter value: {}", value);
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_manager() {
        let binding = ParameterManager::get_manager();
        let mut p_params = binding.lock().unwrap();

        let callback_w = |key: String, value: String| {
            println!("callback(param*)): [{}] = {}", key, value);
        };

        let _callback_id_w = p_params.register_callback("param*", callback_w);

        p_params.set_parameter("paramA", "ABC");
        assert_eq!(p_params.get_parameter("paramA", "HOGE"), "ABC");

        p_params.set_parameter_bool("paramB", true);
        assert_eq!(p_params.get_parameter("paramB", "false"), "true");
        assert_eq!(p_params.get_parameter_bool("paramB", false), true);

        p_params.set_parameter_int("paramC", 1);
        assert_eq!(p_params.get_parameter_int("paramC", 0), 1);
        assert_eq!(p_params.get_parameter_int("paramD", -1), -1);

        // read only
        p_params.set_parameter_int("ro.paramD", 1);
        assert_eq!(p_params.get_parameter_int("ro.paramD", 0), 1);
        p_params.set_parameter_int("ro.paramD", 2);
        assert_eq!(p_params.get_parameter_int("ro.paramD", 0), 1);

        // register callback
        let _callback_2 = |key: String, value: String| {
            println!("callback(exact match): [{}] = {}", key, value);
        };
    }
}

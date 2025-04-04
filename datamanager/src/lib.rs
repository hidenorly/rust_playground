/*
  Copyright (C) 2025 hidenorly

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use std::str::FromStr;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use std::io::{BufRead, Write};


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

    pub fn set_parameter<T: ToString>(&mut self, key: &str, value: T) {
        let mut value = value.to_string().trim().to_string();
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

        if key.ends_with('*') {
            // wild card case
            let _key = key[..key.len() - 1].to_string();
            self.wild_card_listeners
                .entry(_key.clone())
                .or_insert_with(Vec::new)
                .push(listener.clone());
            self.listener_id_reverse.insert(listener_id, key.to_string());
        } else {
            // complete match case
            self.listeners
                .entry(key.to_string())
                .or_insert_with(Vec::new)
                .push(listener.clone());
            self.listener_id_reverse.insert(listener_id, key.to_string());
        }

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

    pub fn get_parameter<T, U>(&self, key: &str, default_value: U) -> T
    where
        T: FromStr + Default,
        U: Into<T>,
    {
        self.params
            .get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| default_value.into())
    }

    pub fn get_parameter_string(&self, key: &str, default_value: &str) -> String{
        return self.get_parameter::<String, &str>(key, default_value);
    }

    pub fn get_parameter_int(&self, key: &str, default_value: i32) -> i32{
        self.params
            .get(key)
            .and_then(|v| v.parse::<f64>().ok().map(|f| f as i32))
            .unwrap_or(default_value)
    }

    pub fn get_parameter_float(&self, key: &str, default_value: f32) -> f32{
        return self.get_parameter::<f32, f32>(key, default_value);
    }

    pub fn get_parameter_bool(&self, key: &str, default_value: bool) -> bool{
        if self.get_parameter_string(key, &default_value.to_string()) == "true" {
            true
        } else {
            false
        }
    }

    pub fn set_parameter_rule(&mut self, key: &str, rule: ParamRule) {
        self.param_rules.insert(key.to_string(), rule);
    }

    pub fn get_parameter_rule(&self, key: &str) -> ParamRule {
        self.param_rules.get(key).cloned().unwrap_or_else(|| ParamRule {
            param_type: ParamType::TypeString,
            range: ParamRange::RangeAny,
            range_min: 0.0,
            range_max: 0.0,
            enum_vals: HashSet::new(),
        })
    }

    pub fn store_to_stream<W: Write>(&self, writer: &mut W) -> bool {
        let mut result = false;
        for (key, value) in &self.params {
            let buf = format!("\"{}\":\"{}\"\n", key, value);
            if writer.write_all(buf.as_bytes()).is_ok() {
                result = true;
            }
        }
        result
    }

    pub fn restore_from_stream<R: BufRead>(&mut self, reader: &mut R, override_existing: bool) -> bool {
        let mut result = false;
        let mut line = String::new();

        while reader.read_line(&mut line).is_ok() && !line.is_empty() {
            let tokens: Vec<&str> = line.trim().split("\":\"").collect();
            if tokens.len() == 2 {
                let key = tokens[0].trim_matches('"').to_string();
                let value = tokens[1].trim_matches('"').to_string();
                if override_existing || !self.params.contains_key(&key) {
                    self.set_parameter(&key, value);
                }
                result = true;
            }
            line.clear(); // Reset line buffer for next iteration
        }

        result
    }
}

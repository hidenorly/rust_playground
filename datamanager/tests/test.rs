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

use mockall::{mock, predicate::eq};
use datamanager::{ParameterManager, ParamRule, ParamType, ParamRange};


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::io::{Cursor};

    use std::fs::{File, OpenOptions};
    use std::io::{BufReader, BufWriter, Write};
    use tempfile::tempdir;

    use super::*;

    mock! {
        MyCallback {
            fn call(&self, key: String, value: String);
        }
    }


    #[test]
    fn test_parameter_manager() {
        let binding = ParameterManager::get_manager();
        let mut p_params = binding.lock().unwrap();

        let callback_w = |key: String, value: String| {
            println!("callback(param*)): [{}] = {}", key, value);
        };

        let _callback_id_w = p_params.register_callback("param*", callback_w);

        p_params.set_parameter("paramA", "ABC");
        assert_eq!(p_params.get_parameter::<String, &str>("paramA", "HOGE"), "ABC");

        p_params.set_parameter("paramB", true);
        assert_eq!(p_params.get_parameter::<String, &str>("paramB", "false"), "true");
        assert_eq!(p_params.get_parameter::<bool, bool>("paramB", false), true);

        p_params.set_parameter("paramC", 1);
        assert_eq!(p_params.get_parameter::<i32, i32>("paramC", 0), 1);
        assert_eq!(p_params.get_parameter::<i32, i32>("paramD", -1), -1);

        let int_value:i32 = p_params.get_parameter("paramC", 0);
        assert_eq!(int_value, 1);

        let int_value2:i32 = p_params.get_parameter_int("paramC", 0);
        assert_eq!(int_value2, 1);

        // check helper func
        p_params.set_parameter("paramE", "1.5");
        assert_eq!(p_params.get_parameter_string("paramE", ""), "1.5");
        assert_eq!(p_params.get_parameter_int("paramE", 0), 1);
        assert_eq!(p_params.get_parameter_float("paramE", 0.0), 1.5);
        assert_eq!(p_params.get_parameter_bool("paramE", true), false);


        // read only
        p_params.set_parameter("ro.paramD", 1);
        assert_eq!(p_params.get_parameter::<i32, i32>("ro.paramD", 0), 1);
        p_params.set_parameter("ro.paramD", 2);
        assert_eq!(p_params.get_parameter::<i32, i32>("ro.paramD", 0), 1);

        // register callback
        let _callback_2 = |key: String, value: String| {
            println!("callback(exact match): [{}] = {}", key, value);
        };
    }

    #[test]
    fn test_callback() {
        let binding = ParameterManager::get_manager();
        let mut p_params = binding.lock().unwrap();

        let mut mock_my_callback = MockMyCallback::new();
        mock_my_callback.expect_call()
            .with(eq("paramA".to_string()), eq("test_value".to_string()))
            .times(1)
            .returning(|_, _| ());

        let mock_callback = move |key: String, value: String| {
            mock_my_callback.call(key, value);
        };

        let _callback_id = p_params.register_callback("paramA", mock_callback);

        p_params.set_parameter("paramA", "test_value");

    }


    #[test]
    fn test_rule_int() {
        let mut manager = ParameterManager::new();
        let rule = ParamRule {
            param_type: ParamType::TypeInt,
            range: ParamRange::Ranged,
            range_min: 1.0,
            range_max: 10.0,
            enum_vals: HashSet::new(),
        };
        
        manager.set_parameter_rule("example", rule.clone());
        let retrieved_rule = manager.get_parameter_rule("example");
        
        match retrieved_rule.param_type {
            ParamType::TypeInt => println!("Retrieved rule type: TypeInt"),
            _ => println!("Retrieved rule type: Other"),
        }

        manager.set_parameter("example", "1");
        assert_eq!(manager.get_parameter_int("example", 0), 1);

        // illegal case then the request should be clamped
        manager.set_parameter("example", "0");
        assert_eq!(manager.get_parameter_int("example", 0), 1);
        manager.set_parameter("example", 0);
        assert_eq!(manager.get_parameter_int("example", 0), 1);
        manager.set_parameter("example", 11);
        assert_eq!(manager.get_parameter_int("example", 0), 10);

        manager.set_parameter("example", 1.0);
        assert_eq!(manager.get_parameter_int("example", 0), 1);

        manager.set_parameter("example", 11.0);
        assert_eq!(manager.get_parameter_int("example", 0), 10);
    }

    #[test]
    fn test_rule_float() {
        let mut manager = ParameterManager::new();
        let rule = ParamRule {
            param_type: ParamType::TypeFloat,
            range: ParamRange::Ranged,
            range_min: -1.0,
            range_max: 1.0,
            enum_vals: HashSet::new(),
        };
        
        manager.set_parameter_rule("example", rule.clone());
        let retrieved_rule = manager.get_parameter_rule("example");
        
        match retrieved_rule.param_type {
            ParamType::TypeFloat => println!("Retrieved rule type: TypeFloat"),
            _ => println!("Retrieved rule type: Other"),
        }

        manager.set_parameter("example", 1.0);
        assert_eq!(manager.get_parameter_bool("example", true), false);
        assert_eq!(manager.get_parameter_string("example", ""), "1"); // Note the return is "1". not "1.0"
        assert_eq!(manager.get_parameter_int("example", 0), 1);
        assert_eq!(manager.get_parameter_float("example", 0.0), 1.0);

        manager.set_parameter("example", 1.1);
        assert_eq!(manager.get_parameter_float("example", 0.0), 1.0);
        manager.set_parameter("example", -1.1);
        assert_eq!(manager.get_parameter_float("example", 0.0), -1.0);
    }

    #[test]
    fn test_rule_bool() {
        let mut manager = ParameterManager::new();
        let rule = ParamRule {
            param_type: ParamType::TypeBool,
            range: ParamRange::Ranged,
            range_min: -1.0,
            range_max: 0.0,
            enum_vals: HashSet::new(),
        };
        
        manager.set_parameter_rule("example", rule.clone());
        let retrieved_rule = manager.get_parameter_rule("example");
        
        match retrieved_rule.param_type {
            ParamType::TypeBool => println!("Retrieved rule type: TypeBool"),
            _ => println!("Retrieved rule type: Other"),
        }

        manager.set_parameter("example", "true");
        assert_eq!(manager.get_parameter_bool("example", false), true);
        assert_eq!(manager.get_parameter_string("example", "false"), "true");
        assert_eq!(manager.get_parameter_int("example", 0), 0); // int("true")==0
        assert_eq!(manager.get_parameter_float("example", 0.0), 0.0); // int("true")==0

        manager.set_parameter("example", "false");
        assert_eq!(manager.get_parameter_bool("example", true), false);
        assert_eq!(manager.get_parameter_string("example", "true"), "false");
        assert_eq!(manager.get_parameter_int("example", 0), 0); // float("true")==0
        assert_eq!(manager.get_parameter_float("example", 0.0), 0.0); // int("true")==0
    }

    #[test]
    fn test_rule_string() {
        let mut manager = ParameterManager::new();
        let rule = ParamRule {
            param_type: ParamType::TypeString,
            range: ParamRange::RangeAny,
            range_min: 0.0,
            range_max: 0.0,
            enum_vals: HashSet::new(),
        };
        
        manager.set_parameter_rule("example", rule.clone());
        let retrieved_rule = manager.get_parameter_rule("example");
        
        match retrieved_rule.param_type {
            ParamType::TypeString => println!("Retrieved rule type: TypeString"),
            _ => println!("Retrieved rule type: Other"),
        }

        manager.set_parameter("example", "low");
        assert_eq!(manager.get_parameter_string("example", ""), "low");
        assert_eq!(manager.get_parameter_int("example", 0), 0);
        assert_eq!(manager.get_parameter_float("example", 0.0), 0.0);
        assert_eq!(manager.get_parameter_bool("example", true), false);

        manager.set_parameter("example", "1.0");
        assert_eq!(manager.get_parameter_int("example", 0), 1);
        assert_eq!(manager.get_parameter_float("example", 0.0), 1.0);
        assert_eq!(manager.get_parameter_bool("example", true), false);

        manager.set_parameter("example", "1");
        assert_eq!(manager.get_parameter_int("example", 0), 1);
        assert_eq!(manager.get_parameter_float("example", 0.0), 1.0);
        assert_eq!(manager.get_parameter_bool("example", true), false);
    }

    #[test]
    fn test_rule_enum() {
        let mut manager = ParameterManager::new();
        let rule = ParamRule {
            param_type: ParamType::TypeString,
            range: ParamRange::RangeEnum,
            range_min: 0.0,
            range_max: 0.0,
            enum_vals: ["low", "mid", "high"].iter().map(|s| s.to_string()).collect(),
        };
        
        manager.set_parameter_rule("example", rule.clone());
        let retrieved_rule = manager.get_parameter_rule("example");
        
        match retrieved_rule.param_type {
            ParamType::TypeString => println!("Retrieved rule type: TypeString"),
            _ => println!("Retrieved rule type: Other"),
        }

        manager.set_parameter("example", "low");
        assert_eq!(manager.get_parameter_string("example", ""), "low");

        manager.set_parameter("example", "mid");
        assert_eq!(manager.get_parameter_string("example", ""), "mid");

        manager.set_parameter("example", "high");
        assert_eq!(manager.get_parameter_string("example", ""), "high");

        manager.set_parameter("example", "off");
        assert_eq!(manager.get_parameter_string("example", ""), "high");
    }

    #[test]
    fn test_store_to_stream() {
        let mut manager = ParameterManager::new();
        manager.set_parameter("key1", "value1");
        manager.set_parameter("key2", "value2");

        let mut output = Vec::new();
        let result = manager.store_to_stream(&mut output);
        assert!(result, "store_to_stream should return true when writing succeeds");

        let output_str = String::from_utf8(output).expect("Failed to convert to string");
        let expected_output1 = "\"key1\":\"value1\"\n\"key2\":\"value2\"\n";
        let expected_output2 = "\"key2\":\"value2\"\n\"key1\":\"value1\"\n";

        assert!(
            output_str == expected_output1 || output_str == expected_output2,
            "Unexpected output: {}",
            output_str
        );
    }

    #[test]
    fn test_restore_from_stream_override() {
        let input_data = "\"key1\":\"value1\"\n\"key2\":\"value2\"\n";
        let cursor = Cursor::new(input_data.as_bytes());
        let mut reader = BufReader::new(cursor);

        let mut manager = ParameterManager::new();
        let result = manager.restore_from_stream(&mut reader, true);
        assert!(result, "restore_from_stream should return true when successful");

        assert_eq!(
            manager.get_parameter_string("key1", ""),
            "value1",
            "key1 should be set to value1"
        );
        assert_eq!(
            manager.get_parameter_string("key2", ""),
            "value2",
            "key2 should be set to value2"
        );
    }

    #[test]
    fn test_restore_from_stream_no_override() {
        let input_data = "\"key1\":\"new_value\"\n\"key2\":\"new_value2\"\n";
        let cursor = Cursor::new(input_data.as_bytes());
        let mut reader = BufReader::new(cursor);

        let mut manager = ParameterManager::new();
        manager.set_parameter("key1", "old_value");

        let result = manager.restore_from_stream(&mut reader, false);
        assert!(result, "restore_from_stream should return true when successful");

        assert_eq!(
            manager.get_parameter_string("key1", ""),
            "old_value",
            "key1 should retain old_value because override is false"
        );
        assert_eq!(
            manager.get_parameter_string("key2", ""),
            "new_value2",
            "key2 should be set to new_value2"
        );
    }

    #[test]
    fn test_restore_from_stream_empty_input() {
        let input_data = "";
        let cursor = Cursor::new(input_data.as_bytes());
        let mut reader = BufReader::new(cursor);

        let mut manager = ParameterManager::new();
        let result = manager.restore_from_stream(&mut reader, true);
        assert!(!result, "restore_from_stream should return false on empty input");
    }

    #[test]
    fn test_restore_from_stream_malformed_input() {
        let input_data = "\"key1\" \"value1\"\n\"key2\":\"value2\"\n"; // invalid format on key1
        let cursor = Cursor::new(input_data.as_bytes());
        let mut reader = BufReader::new(cursor);

        let mut manager = ParameterManager::new();
        let result = manager.restore_from_stream(&mut reader, true);
        assert!(result, "restore_from_stream should return true if at least one line is valid");

        assert!(
            manager.get_parameter_string("key1", "") == "",
            "Malformed key1 should not be added"
        );
        assert_eq!(
            manager.get_parameter_string("key2", ""),
            "value2",
            "Valid key2 should be added"
        );
    }

    #[test]
    fn test_store_to_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("params.txt");
        let file = File::create(&file_path).expect("Failed to create file");
        let mut writer = BufWriter::new(file);

        let mut manager = ParameterManager::new();
        manager.set_parameter("key1", "value1");
        manager.set_parameter("key2", "value2");

        assert!(manager.store_to_stream(&mut writer), "Failed to store to stream");
    }

    #[test]
    fn test_restore_from_file() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("params.txt");
        {
            let file = File::create(&file_path).expect("Failed to create file");
            let mut writer = BufWriter::new(file);
            writeln!(writer, "\"key1\":\"value1\"\n\"key2\":\"value2\"").expect("Failed to write");
        }

        let file = File::open(&file_path).expect("Failed to open file");
        let mut reader = BufReader::new(file);
        let mut manager = ParameterManager::new();

        assert!(manager.restore_from_stream(&mut reader, true), "Failed to restore from file");
        assert_eq!(manager.get_parameter_string("key1", ""), "value1");
        assert_eq!(manager.get_parameter_string("key2", ""), "value2");
    }

    #[test]
    fn test_restore_from_file_no_override() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("params.txt");
        {
            let file = File::create(&file_path).expect("Failed to create file");
            let mut writer = BufWriter::new(file);
            writeln!(writer, "\"key1\":\"new_value\"\n\"key2\":\"value2\"").expect("Failed to write");
        }

        let file = File::open(&file_path).expect("Failed to open file");
        let mut reader = BufReader::new(file);
        let mut manager = ParameterManager::new();
        manager.set_parameter("key1", "old_value");

        assert!(manager.restore_from_stream(&mut reader, false), "Failed to restore with no override");
        assert_eq!(manager.get_parameter_string("key1", ""), "old_value");
        assert_eq!(manager.get_parameter_string("key2", ""), "value2");
    }
}


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
}

use std::collections::BTreeSet;

use toml::{value::Array, Table, Value};

pub enum ProcessResult {
    Flatten(toml::Value),
    Empty,
    NotEmpty,
}

fn merge_prefix(table: &mut toml::Table, prefix: &str, new_prefix: &str) {
    let prefix_keys = table
        .keys()
        .filter(|k| k.starts_with(prefix))
        .cloned()
        .collect::<Vec<_>>();

    if prefix_keys.is_empty() {
        return;
    }
    if prefix_keys.len() == 1 {
        dbg!(&prefix_keys);
    }

    let mut new_table = Table::new();

    for k in prefix_keys.iter() {
        let (_, suffix) = k.split_once(prefix).unwrap();
        new_table.insert(format!("{new_prefix}{suffix}"), table.remove(k).unwrap());
    }

    dbg!(&new_table);

    table.insert(prefix.to_string(), new_table.into());
}

fn replace_boolean(table: &mut toml::Table, key: &str) {
    if let Some(v) = table.get_mut(key) {
        match v {
            Value::String(s) if s == "1" => *v = true.into(),
            Value::String(s) if s == "0" => *v = false.into(),
            Value::Integer(1) => *v = true.into(),
            Value::Integer(0) => *v = false.into(),
            _ => (),
        }
    }
}

fn replace_num(table: &mut toml::Table, key: &str) {
    if let Some(v) = table.get_mut(key) {
        if let Some(num) = v.as_str().and_then(|v| v.parse::<i64>().ok()) {
            *v = toml::Value::Integer(num)
        }
    }
}

fn flatten_into_array(tbl: &mut toml::Table) -> Option<Array> {
    let keys = tbl
        .keys()
        .map(|num| num.parse())
        .collect::<Result<BTreeSet<usize>, _>>();
    if let Ok(num_keys) = keys {
        if num_keys.iter().any(|&num| num > 100) {
            return None;
        }

        let no_gaps = num_keys
            .iter()
            .zip(num_keys.iter().skip(1))
            .all(|(&a, &b)| a + 1 == b);
        // The table has only numeric keys convert it into an array
        if no_gaps {
            let arr: Array = num_keys
                .iter()
                .map(|num| tbl.remove(&num.to_string()).unwrap())
                .collect();

            return Some(arr);
        }
    }

    None
}

fn flatten_singular(tbl: &mut toml::Table, key: &str) -> Option<toml::Value> {
    if tbl.len() != 1 {
        return None;
    }

    tbl.remove(key)
}

pub fn process_skill_value(table: &mut toml::Table) -> ProcessResult {
    table.remove("origin");
    table.remove("z");
    table.remove("mob");
    table.remove("hit");
    table.remove("summon");
    table.remove("a0");
    table.remove("a1");

    replace_boolean(table, "disabled");
    replace_boolean(table, "disable");
    replace_boolean(table, "hitOnce");
    replace_boolean(table, "invisible");
    replace_boolean(table, "timeLimited");
    //replace_num(table, "time");

    merge_prefix(table, "action", "Action_");
    merge_prefix(table, "skill", "Skill_");
    merge_prefix(table, "effect", "Effect_");

    if let Some(delay) = flatten_singular(table, "delay") {
        return ProcessResult::Flatten(delay);
    }

    table.retain(|_, v| {
        if let Some(tbl) = v.as_table_mut() {
            match process_skill_value(tbl) {
                ProcessResult::Flatten(new_v) => {
                    *v = new_v;
                    true
                }
                ProcessResult::Empty => false,
                ProcessResult::NotEmpty => true,
            }
        } else {
            true
        }
    });

    if table.is_empty() {
        return ProcessResult::Empty;
    }

    if let Some(arr) = flatten_into_array(table) {
        return ProcessResult::Flatten(arr.into());
    }

    ProcessResult::NotEmpty
}

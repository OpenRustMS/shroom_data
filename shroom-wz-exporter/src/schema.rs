use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt,
};

use anyhow::Context;
use convert_case::{Case, Casing};
use quote::{__private::TokenStream, format_ident};
use toml::{Table, Value};

pub type SchemaRef = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaValue {
    Bool,
    Int,
    Float,
    Vec2,
    String,
    Struct(SchemaRef),
    NumericDir(Box<SchemaValue>),
    Optional(Box<SchemaValue>),
    Array(Box<SchemaValue>),
}

impl SchemaValue {
    pub fn into_optional(self) -> SchemaValue {
        SchemaValue::Optional(Box::new(self))
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, SchemaValue::Optional(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, SchemaValue::Array(_))
    }

    pub fn make_optional(&mut self) {
        if self.is_array() {
            return;
        }
        if !self.is_optional() {
            let opt = self.clone().into_optional();
            *self = opt;
        }
    }
}

fn check_vec2(tbl: &Table) -> bool {
    if tbl.keys().len() != 2 {
        return false;
    }

    tbl.contains_key("x") && tbl.contains_key("y")
}

fn check_numeric(tbl: &Table) -> bool {
    let mut nums = HashSet::new();
    for k in tbl.keys() {
        if let Ok(num) = k.parse::<i64>() {
            if !nums.insert(num) {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

impl SchemaValue {
    pub fn merge_with(self, r: Option<&Self>) -> anyhow::Result<Self> {
        Ok(match (self, r) {
            (SchemaValue::Optional(v), None) => SchemaValue::Optional(v),
            (val, None) => val.into_optional(),
            (SchemaValue::Optional(l), Some(r)) if l.as_ref() == r => SchemaValue::Optional(l),
            (SchemaValue::Array(l), Some(SchemaValue::Array(r))) => {
                SchemaValue::Array(Box::new(SchemaValue::merge_with(*l, Some(r))?))
            }
            (SchemaValue::Optional(l), Some(SchemaValue::Optional(r))) => {
                SchemaValue::Optional(Box::new(SchemaValue::merge_with(*l, Some(r))?))
            }

            (l, Some(r)) if &l == r => l,
            (l, r) => anyhow::bail!("Invalid merge: {:?} {:?}", l, r),
        })
    }

    pub fn from_serde_val(val: &Value, key: Option<&str>) -> Self {
        match val {
            Value::String(_) => SchemaValue::String,
            Value::Integer(_) => SchemaValue::Int,
            Value::Float(_) => SchemaValue::Float,
            Value::Boolean(_) => SchemaValue::Bool,
            Value::Datetime(_) => todo!(),
            Value::Array(arr) => {
                let first = arr.first().unwrap();
                SchemaValue::Array(Box::new(SchemaValue::from_serde_val(first, key)))
            }
            Value::Table(tbl) if check_vec2(tbl) => SchemaValue::Vec2,
            Value::Table(tbl) if check_numeric(tbl) => {
                let first = tbl.values().next().unwrap();
                let val = SchemaValue::from_serde_val(first, key);
                SchemaValue::NumericDir(Box::new(val))
            }
            Value::Table(_tbl) => {
                let key = key.unwrap();
                let name = fmt_type_name(key);
                SchemaValue::Struct(name)
            }
        }
    }

    pub fn to_rust_type(&self) -> Cow<'_, str> {
        match self {
            SchemaValue::Float => "f32".into(),
            SchemaValue::Int => "i64".into(),
            SchemaValue::String => "String".into(),
            SchemaValue::Vec2 => "Vec2".into(),
            SchemaValue::Bool => "bool".into(),
            SchemaValue::Array(v) => format!("Vec<{}>", v.to_rust_type()).into(),
            SchemaValue::Struct(name) => name.as_str().into(),
            SchemaValue::NumericDir(name) => {
                format!("BTreeMap<i64, {}>", name.to_rust_type()).into()
            }
            SchemaValue::Optional(opt) => {
                let ty = opt.to_rust_type();
                format!("Option<{ty}>").into()
            }
        }
    }

    pub fn to_rust_type_token(&self) -> TokenStream {
        match self {
            SchemaValue::Float => quote::quote!(f32),
            SchemaValue::Int => quote::quote!(i64),
            SchemaValue::String => quote::quote!(String),
            SchemaValue::Vec2 => quote::quote!(Vec2),
            SchemaValue::Bool => quote::quote!(bool),
            SchemaValue::Array(name) => {
                let id = name.to_rust_type_token();
                quote::quote!(Vec<#id>)
            }
            SchemaValue::Struct(name) => {
                let id = if name.parse::<usize>().is_ok() {
                    format_ident!("_{name}")
                } else {
                    format_ident!("{name}")
                };
                quote::quote!(#id)
            }
            SchemaValue::NumericDir(name) => {
                let id = name.to_rust_type_token();
                quote::quote!(BTreeMap<i64, #id>)
            }

            SchemaValue::Optional(opt) => {
                let ty = opt.to_rust_type_token();
                quote::quote!(Option<#ty>)
            }
        }
    }
}

fn fmt_type_name(s: &str) -> String {
    let s = s.to_case(Case::Pascal);
    if s.parse::<usize>().is_ok() {
        format!("_{s}")
    } else {
        s
    }
}

fn fmt_field_name(s: &str) -> String {
    let s = s.to_case(Case::Snake);
    if s.parse::<usize>().is_ok() {
        format!("_{s}")
    } else {
        s
    }
}

#[derive(Debug)]
pub struct SchemaStruct(HashMap<String, SchemaValue>);

impl SchemaStruct {
    pub fn has_optional(&self) -> bool {
        self.0.values().any(|f| f.is_optional())
    }

    pub fn merge_fields(&mut self, other: SchemaStruct) -> anyhow::Result<()> {
        // Mark keys in current struct as optional
        for (k, v) in self.0.iter_mut() {
            *v = v.clone().merge_with(other.0.get(k)).context(k.clone())?;
        }

        // Add non-existing keys from the other schema
        for (k, mut v) in other.0.into_iter() {
            self.0.entry(k).or_insert_with(|| {
                v.make_optional();
                v
            });
        }

        Ok(())
    }

    pub fn fmt_rust_struct(&self, name: &str, mut w: impl fmt::Write) -> fmt::Result {
        let name = fmt_type_name(name);
        writeln!(w, "#[derive(Debug)]")?;
        writeln!(w, "pub struct {name} {{")?;
        for (name, val) in self.0.iter() {
            let name = fmt_field_name(name);
            let ty = val.to_rust_type();
            writeln!(w, "\tpub {name}: {ty},")?;
        }
        writeln!(w, "}}")?;

        Ok(())
    }

    pub fn from_serde(dir: &Table) -> Self {
        let mut fields = HashMap::new();

        for (k, v) in dir.iter() {
            let key = k;
            let ty = SchemaValue::from_serde_val(v, Some(key));
            fields.insert(k.to_string(), ty);
        }

        Self(fields)
    }
}

#[derive(Debug)]
pub struct Schema {
    schema_structs: HashMap<String, SchemaStruct>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            schema_structs: HashMap::new(),
        }
    }
    pub fn from_multiple_roots_dir<'a>(
        root_name: &str,
        dirs: impl Iterator<Item = &'a Table>,
    ) -> Self {
        let mut schema = Self {
            schema_structs: HashMap::new(),
        };

        for dir in dirs {
            schema.process_dir(root_name, dir).unwrap();
        }

        schema
    }

    pub fn from_root_dir(root_name: &str, dir: &Table) -> Self {
        let mut schema = Self {
            schema_structs: HashMap::new(),
        };

        schema.process_dir(root_name, dir).unwrap();

        schema
    }

    fn process_num_dir(&mut self, name: &str, dir: &Table) -> anyhow::Result<()> {
        //TODO support double-nesting for Footholds for example fh/1/1
        for (_k, v) in dir.iter() {
            match v {
                Value::Table(tbl) if check_vec2(tbl) => (),
                Value::Table(tbl) if check_numeric(tbl) => self.process_num_dir(name, &tbl)?,
                Value::Table(tbl) => self.process_dir(name, &tbl)?,
                Value::Array(arr) => {
                    for v in arr {
                        if let Value::Table(tbl) = v {
                            self.process_dir(name, &tbl)?;
                        }
                    }
                }
                _ => (),
            }
        }

        Ok(())
    }

    pub fn process_dir(&mut self, name: &str, dir: &Table) -> anyhow::Result<()> {
        let strct = SchemaStruct::from_serde(dir);
        let name = fmt_type_name(name);

        //Either insert or merge
        if let Some(merge_strct) = self.schema_structs.get_mut(&name) {
            merge_strct.merge_fields(strct)?;
        } else {
            self.schema_structs.insert(name.to_string(), strct);
        }

        // Process all sub structs
        for (k, v) in dir.iter() {
            match v {
                Value::Table(tbl) if check_vec2(tbl) => (),
                Value::Table(tbl) if check_numeric(tbl) => self.process_num_dir(k, &tbl)?,
                Value::Table(tbl) => self.process_dir(k, &tbl)?,
                Value::Array(arr) => {
                    for v in arr {
                        if let Value::Table(tbl) = v {
                            self.process_dir(k, &tbl)?;
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn to_code(&self) -> String {
        let mut s = String::new();

        for (name, strct) in self.schema_structs.iter() {
            strct.fmt_rust_struct(name, &mut s).unwrap();
            s.push('\n');
        }

        s
    }
}

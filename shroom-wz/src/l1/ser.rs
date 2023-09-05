use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

use serde::{
    ser::{SerializeMap, SerializeStruct},
    Serialize,
};

use crate::file::{WzIO, WzImgReader};

use super::{
    obj::WzObject,
    prop::{WzConvex2D, WzValue, WzVector2D},
};

impl Serialize for WzVector2D {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("vec2", 2)?;
        s.serialize_field("x", &self.x.0)?;
        s.serialize_field("y", &self.y.0)?;
        s.end()
    }
}

impl Serialize for WzConvex2D {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

pub struct WzValueSerializer<'r, R> {
    value: &'r WzValue,
    r: Rc<RefCell<WzImgReader<R>>>,
}

impl<'r, R: WzIO> Serialize for WzValueSerializer<'r, R> {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let WzValueSerializer { r, value } = self;
        match &value {
            WzValue::Null => ser.serialize_none(),
            WzValue::Short1(v) | WzValue::Short2(v) => ser.serialize_i16(*v),
            WzValue::Int1(v) | WzValue::Int2(v) => ser.serialize_i32(v.0),
            WzValue::Long(v) => ser.serialize_i64(v.0),
            WzValue::F32(v) => ser.serialize_f32(v.0),
            WzValue::F64(v) => ser.serialize_f64(*v),
            WzValue::Str(v) => ser.serialize_str(v.as_str().unwrap_or("invalid")),
            WzValue::Obj(obj) => {
                let r = r.clone();
                let object = { r.as_ref().borrow_mut().read_obj(obj).unwrap() };
                let obj_ser = WzObjectSerializer { object: &object, r };
                obj_ser.serialize(ser)
            }
        }
    }
}

pub struct WzObjectSerializer<'r, R> {
    object: &'r WzObject,
    r: Rc<RefCell<WzImgReader<R>>>,
}

impl<'r, R: WzIO> Serialize for WzObjectSerializer<'r, R> {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.object {
            super::obj::WzObject::Property(prop) => {
                let mut s = ser.serialize_map(prop.entries.0.len().into())?;
                for entry in prop.entries.0.iter() {
                    s.serialize_key(entry.name.as_str().unwrap())?;
                    let val_ser = WzValueSerializer {
                        value: &entry.val,
                        r: self.r.clone(),
                    };
                    s.serialize_value(&val_ser)?;
                }
                s.end()
            }
            super::obj::WzObject::Canvas(canvas) => {
                if let Some(ref prop) = canvas.property {
                    let mut s = ser.serialize_map(prop.entries.0.len().into())?;
                    for entry in prop.entries.0.iter() {
                        s.serialize_key(entry.name.as_str().unwrap())?;
                        let val_ser = WzValueSerializer {
                            value: &entry.val,
                            r: self.r.clone(),
                        };
                        s.serialize_value(&val_ser)?;
                    }
                    s.end()
                } else {
                    ser.serialize_none()
                }
            }
            super::obj::WzObject::UOL(_) => ser.serialize_none(),
            super::obj::WzObject::Vec2(vec) => vec.serialize(ser),
            super::obj::WzObject::Convex2D(vex) => vex.serialize(ser),
            super::obj::WzObject::SoundDX8(_) => ser.serialize_none(),
        }
    }
}

pub struct WzImgSerializer<R> {
    img_reader: Rc<RefCell<WzImgReader<R>>>,
    root: WzObject,
}

impl<R: WzIO> WzImgSerializer<R> {
    pub fn new(mut img_reader: WzImgReader<R>) -> anyhow::Result<Self> {
        let root = img_reader.read_root_obj()?;
        Ok(Self {
            img_reader: Rc::new(RefCell::new(img_reader)),
            root,
        })
    }
}

impl<'r, R: WzIO> Serialize for WzImgSerializer<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        WzObjectSerializer {
            object: &self.root,
            r: self.img_reader.clone(),
        }
        .serialize(serializer)
    }
}

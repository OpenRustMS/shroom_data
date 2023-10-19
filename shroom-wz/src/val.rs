use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    time::Duration,
};

use derive_more::IsVariant;
use indexmap::IndexMap;

use crate::{
    canvas::Canvas,
    file::{WzIO, WzImgReader},
    l1::{
        canvas::WzCanvas,
        obj::WzObject,
        prop::{WzObj, WzPropValue, WzProperty, WzVector2D},
        sound::WzSound,
    },
};

pub type Map = IndexMap<String, WzValue>;

#[derive(Debug)]
pub struct CanvasVal {
    pub canvas: WzCanvas,
    pub sub: Option<Box<WzValue>>,
}

impl CanvasVal {
    pub fn read_canvas<R: WzIO>(&self, r: &mut WzImgReader<R>) -> anyhow::Result<Canvas> {
        r.read_canvas(&self.canvas)
    }
}

#[derive(Debug)]
pub struct SoundVal {
    pub sound: WzSound,
}

impl SoundVal {
    pub fn read_data<R: WzIO>(&self, r: &mut WzImgReader<R>) -> anyhow::Result<Vec<u8>> {
        r.read_sound(&self.sound)
    }

    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.sound.len_ms.0 as u64)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2Val {
    pub x: i32,
    pub y: i32,
}

impl From<WzVector2D> for Vec2Val {
    fn from(value: WzVector2D) -> Self {
        Self {
            x: value.x.0,
            y: value.y.0,
        }
    }
}

impl Display for Vec2Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x={},y={}", self.x, self.y)
    }
}

#[derive(Debug)]
pub struct Vex2Val(pub Vec<Vec2Val>);

#[derive(Debug)]
pub struct ObjectVal(pub Map);

impl ObjectVal {
    pub fn get(&self, index: &str) -> Option<&WzValue> {
        self.0.get(index)
    }

    pub fn must_get(&self, index: &str) -> anyhow::Result<&WzValue> {
        self.0
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("Missing index {}", index))
    }

    pub fn get_into<'a, T: TryFrom<&'a WzValue>>(
        &'a self,
        index: &str,
    ) -> Result<Option<T>, T::Error> {
        self.0.get(index).map(|v| v.try_into()).transpose()
    }

    pub fn must_get_into<'a, T: TryFrom<&'a WzValue>>(&'a self, index: &str) -> anyhow::Result<T>
    where
        T::Error: std::fmt::Debug,
    {
        Ok(self.must_get(index)?.try_into().map_err(|e| {
            anyhow::anyhow!("Failed to convert {} to {}: {:?}", index, stringify!(T), e)
        })?)
    }
}

impl Index<&str> for ObjectVal {
    type Output = WzValue;

    fn index(&self, index: &str) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<&str> for ObjectVal {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.0.get_mut(index).unwrap()
    }
}

#[derive(Debug, IsVariant)]
pub enum WzValue {
    Object(ObjectVal),
    Null,
    F32(f32),
    F64(f64),
    Short(i16),
    Int(i32),
    Long(i64),
    String(String),
    Vec(Vec2Val),
    Convex(Vex2Val),
    Sound(SoundVal),
    Canvas(CanvasVal),
    Link(String),
}

impl WzValue {
    pub fn get_path(&self, path: &str) -> Option<&WzValue> {
        let mut cur = self;
        for part in path.split('/') {
            let cur_obj = match cur {
                WzValue::Object(v) => v,
                WzValue::Canvas(v) => {
                    // We get the next object from the canvas If there's one
                    if let Some(WzValue::Object(v)) = v.sub.as_deref() {
                        v
                    } else {
                        return None;
                    }
                }
                _ => return None,
            };

            if let Some(v) = cur_obj.0.get(part) {
                cur = v;
            } else {
                return None;
            }
        }

        Some(cur)
    }

    pub fn as_object(&self) -> Option<&ObjectVal> {
        match self {
            WzValue::Object(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        match self {
            WzValue::F32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            WzValue::F64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i16(&self) -> Option<i16> {
        match self {
            WzValue::Short(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            WzValue::Int(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            WzValue::Long(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            WzValue::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_vec(&self) -> Option<&Vec2Val> {
        match self {
            WzValue::Vec(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_convex(&self) -> Option<&Vex2Val> {
        match self {
            WzValue::Convex(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_sound(&self) -> Option<&SoundVal> {
        match self {
            WzValue::Sound(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_canvas(&self) -> Option<&CanvasVal> {
        match self {
            WzValue::Canvas(v) => Some(v),
            _ => None,
        }
    }
}

macro_rules! try_into_val {
    ($ty:ty, $into_fn:ident) => {
        impl TryFrom<&WzValue> for $ty {
            type Error = anyhow::Error;

            fn try_from(v: &WzValue) -> Result<$ty, Self::Error> {
                v.$into_fn()
                    .ok_or_else(|| anyhow::anyhow!("Expected {}, got {:?}", stringify!($ty), v))
            }
        }
    };
    (ref, $ty:ty, $into_fn:ident) => {
        impl<'a> TryFrom<&'a WzValue> for &'a $ty {
            type Error = anyhow::Error;

            fn try_from(v: &'a WzValue) -> Result<&'a $ty, Self::Error> {
                v.$into_fn()
                    .ok_or_else(|| anyhow::anyhow!("Expected {}, got {:?}", stringify!($ty), v))
            }
        }
    };
}

try_into_val!(f32, as_f32);
try_into_val!(f64, as_f64);
try_into_val!(i16, as_i16);
try_into_val!(i32, as_i32);
try_into_val!(i64, as_i64);

try_into_val!(ref, Vec2Val, as_vec);
try_into_val!(ref, str, as_string);
try_into_val!(ref, ObjectVal, as_object);
try_into_val!(ref, SoundVal, as_sound);
try_into_val!(ref, Vex2Val, as_convex);

impl WzValue {
    pub fn read<R: WzIO>(r: &mut WzImgReader<R>) -> anyhow::Result<WzValue> {
        let obj = r.root_obj();
        Self::read_obj(r, &obj)
    }

    fn read_val<R: WzIO>(r: &mut WzImgReader<R>, val: &WzPropValue) -> anyhow::Result<WzValue> {
        Ok(match val {
            WzPropValue::Null => WzValue::Null,
            WzPropValue::Short1(v) | WzPropValue::Short2(v) => WzValue::Short(*v),
            WzPropValue::Int1(v) | WzPropValue::Int2(v) => WzValue::Int(v.0),
            WzPropValue::Long(v) => WzValue::Long(v.0),
            WzPropValue::F32(v) => WzValue::F32(v.0),
            WzPropValue::F64(v) => WzValue::F64(*v),
            WzPropValue::Str(v) => WzValue::String(v.to_string()),
            WzPropValue::Obj(v) => Self::read_obj(r, v)?,
        })
    }

    fn read_prop<R: WzIO>(r: &mut WzImgReader<R>, prop: &WzProperty) -> anyhow::Result<WzValue> {
        let mut map = Map::new();
        for entry in prop.entries.0.iter() {
            let k = entry.name.as_ref().to_string();
            map.insert(k, Self::read_val(r, &entry.val)?);
        }
        Ok(WzValue::Object(ObjectVal(map)))
    }

    fn read_obj<R: WzIO>(r: &mut WzImgReader<R>, obj: &WzObj) -> anyhow::Result<WzValue> {
        let obj = r.read_obj(obj)?;
        Ok(match obj {
            WzObject::Property(prop) => Self::read_prop(r, &prop)?,
            WzObject::Canvas(canvas) => {
                let prop = if let Some(prop) = canvas.property.as_ref() {
                    Some(Box::new(Self::read_prop(r, prop)?))
                } else {
                    None
                };
                WzValue::Canvas(CanvasVal { canvas, sub: prop })
            }
            WzObject::UOL(link) => WzValue::Link(link.entries.as_ref().to_string()),
            WzObject::Vec2(vec2) => WzValue::Vec(vec2.into()),
            WzObject::Convex2D(vex) => {
                WzValue::Convex(Vex2Val(vex.0.iter().map(|v| Vec2Val::from(*v)).collect()))
            }
            WzObject::SoundDX8(sound) => WzValue::Sound(SoundVal {
                sound: sound.clone(),
            }),
        })
    }
}

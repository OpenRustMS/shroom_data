use std::str::FromStr;

use shroom_wz::val::{ObjectVal, Vec2Val, WzValue};

#[derive(Debug)]
pub enum ElementAttribute {
    Fire,
    Ice,
    Poison,
    Holy,
    Light,
    Physical,
    Dark,
}

impl FromStr for ElementAttribute {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "f" => Self::Fire,
            "i" => Self::Ice,
            "s" => Self::Poison,
            "h" => Self::Holy,
            "l" => Self::Light,
            "p" => Self::Physical,
            "d" => Self::Dark,
            _ => anyhow::bail!("Invalid elem attribute: {}", s),
        })
    }
}

impl TryFrom<&WzValue> for ElementAttribute {
    type Error = anyhow::Error;

    fn try_from(val: &WzValue) -> Result<Self, Self::Error> {
        match val {
            WzValue::String(s) => Self::from_str(s),
            _ => Err(anyhow::anyhow!("Expected string, got {:?}", val)),
        }
    }
}

#[derive(Debug)]
pub enum EvalTerm {
    Num(i32),
    Term(String),
}

impl TryFrom<&WzValue> for EvalTerm {
    type Error = anyhow::Error;

    fn try_from(val: &WzValue) -> Result<Self, Self::Error> {
        match val {
            WzValue::Int(i) => Ok(Self::Num(*i)),
            WzValue::String(s) => Ok(Self::Term(s.clone())),
            _ => Err(anyhow::anyhow!("Expected int or string, got {:?}", val)),
        }
    }
}

#[derive(Debug)]
pub struct Skill {
    pub fix_damage: Option<EvalTerm>,
    pub attack_count: Option<EvalTerm>,
    pub mob_count: Option<EvalTerm>,
    pub hp_con: Option<EvalTerm>,
    pub mp_con: Option<EvalTerm>,
    pub damage: Option<EvalTerm>,
    pub mastery: Option<EvalTerm>,
    pub dam_r: Option<EvalTerm>,
    pub dot: Option<EvalTerm>,
    pub dot_time: Option<EvalTerm>,
    pub meso_r: Option<EvalTerm>,
    pub speed: Option<EvalTerm>,
    pub jump: Option<EvalTerm>,
    pub pad: Option<EvalTerm>,
    pub mad: Option<EvalTerm>,
    pub pdd: Option<EvalTerm>,
    pub mdd: Option<EvalTerm>,
    pub eva: Option<EvalTerm>,
    pub acc: Option<EvalTerm>,
    pub hp: Option<EvalTerm>,
    pub mhp_r: Option<EvalTerm>,
    pub mp: Option<EvalTerm>,
    pub mmp_r: Option<EvalTerm>,
    pub prop: Option<EvalTerm>,
    pub sub_prop: Option<EvalTerm>,
    pub cooltime: Option<EvalTerm>,
    pub asr_r: Option<EvalTerm>,
    pub ter_r: Option<EvalTerm>,
    pub emdd: Option<EvalTerm>,
    pub emhp: Option<EvalTerm>,
    pub emmp: Option<EvalTerm>,
    pub epad: Option<EvalTerm>,
    pub epdd: Option<EvalTerm>,
    pub cr: Option<EvalTerm>,
    pub t: Option<EvalTerm>,
    pub u: Option<EvalTerm>,
    pub v: Option<EvalTerm>,
    pub w: Option<EvalTerm>,
    pub x: Option<EvalTerm>,
    pub y: Option<EvalTerm>,
    pub z: Option<EvalTerm>,
    pub pad_r: Option<EvalTerm>,
    pub pad_x: Option<EvalTerm>,
    pub mad_r: Option<EvalTerm>,
    pub mad_x: Option<EvalTerm>,
    pub pdd_r: Option<EvalTerm>,
    pub mdd_r: Option<EvalTerm>,
    pub eva_r: Option<EvalTerm>,
    pub acc_r: Option<EvalTerm>,
    pub ignore_mob_pdp_r: Option<EvalTerm>,
    pub ignore_mob_dam_r: Option<EvalTerm>,
    pub critical_damage_min: Option<EvalTerm>,
    pub critical_damage_max: Option<EvalTerm>,
    pub exp_r: Option<EvalTerm>,
    pub er: Option<EvalTerm>,
    pub ar: Option<EvalTerm>,
    pub pd_r: Option<EvalTerm>,
    pub md_r: Option<EvalTerm>,
    pub psd_jump: Option<EvalTerm>,
    pub psd_speed: Option<EvalTerm>,
    pub elem_attr: Option<ElementAttribute>,
    pub range: Option<(Vec2Val, Vec2Val)>,
}

impl Skill {
    pub fn from_value(id: u32, val: &ObjectVal) -> anyhow::Result<Skill> {
        let c: &ObjectVal = val.must_get_into("common")?;

        if let Ok(level) = val.must_get_into::<&ObjectVal>("level") {
            dbg!(id);
        }

        //dbg!(&c.0.keys().collect::<Vec<_>>());

        let range = if let Some(lt) = c.get("lt") {
            let lt: &Vec2Val = lt.try_into()?;
            let rb: &Vec2Val = c.must_get_into("rb")?;
            Some((lt.clone(), rb.clone()))
        } else {
            None
        };

        Ok(Skill {
            fix_damage: c.get_into("fixdamage")?,
            attack_count: c.get_into("attackcount")?,
            mob_count: c.get_into("mobcount")?,
            hp_con: c.get_into("hpcon")?,
            mp_con: c.get_into("mpcon")?,
            damage: c.get_into("damage")?,
            mastery: c.get_into("mastery")?,
            dam_r: c.get_into("damr")?,
            dot: c.get_into("dot")?,
            dot_time: c.get_into("dottime")?,
            meso_r: c.get_into("mesor")?,
            speed: c.get_into("speed")?,
            jump: c.get_into("jump")?,
            pad: c.get_into("pad")?,
            mad: c.get_into("mad")?,
            pdd: c.get_into("pdd")?,
            mdd: c.get_into("mdd")?,
            eva: c.get_into("eva")?,
            acc: c.get_into("acc")?,
            hp: c.get_into("hp")?,
            mhp_r: c.get_into("mhpr")?,
            mp: c.get_into("mp")?,
            mmp_r: c.get_into("mmpr")?,
            prop: c.get_into("prop")?,
            sub_prop: c.get_into("subprop")?,
            cooltime: c.get_into("cooltime")?,
            asr_r: c.get_into("asrr")?,
            ter_r: c.get_into("terr")?,
            emdd: c.get_into("emdd")?,
            emhp: c.get_into("emhp")?,
            emmp: c.get_into("emmp")?,
            epad: c.get_into("epad")?,
            epdd: c.get_into("epdd")?,
            cr: c.get_into("cr")?,
            t: c.get_into("t")?,
            u: c.get_into("u")?,
            v: c.get_into("v")?,
            w: c.get_into("w")?,
            x: c.get_into("x")?,
            y: c.get_into("y")?,
            z: c.get_into("z")?,
            pad_r: c.get_into("padr")?,
            pad_x: c.get_into("padx")?,
            mad_r: c.get_into("madr")?,
            mad_x: c.get_into("madx")?,
            pdd_r: c.get_into("pddr")?,
            mdd_r: c.get_into("mddr")?,
            eva_r: c.get_into("evar")?,
            acc_r: c.get_into("accr")?,
            ignore_mob_pdp_r: c.get_into("impr")?,
            ignore_mob_dam_r: c.get_into("imdr")?,
            critical_damage_min: c.get_into("cdmin")?,
            critical_damage_max: c.get_into("cdmax")?,
            exp_r: c.get_into("expr")?,
            er: c.get_into("er")?,
            ar: c.get_into("ar")?,
            pd_r: c.get_into("pdr")?,
            md_r: c.get_into("mdr")?,
            psd_jump: c.get_into("psdjump")?,
            psd_speed: c.get_into("psdspeed")?,
            elem_attr: val.get_into("elemAttr")?,
            range,
        })
    }
}

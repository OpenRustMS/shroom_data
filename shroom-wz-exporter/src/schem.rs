use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug)]
pub struct Fly;

#[derive(Debug)]
pub struct Summoned;

#[derive(Debug)]
pub struct PsdSkill {
    pub data: BTreeMap<i64, Data>,
    pub extra: BTreeMap<i64, Extra>,
}

#[derive(Debug)]
pub struct Knuckle {
    pub fist: Option<Fist>,
    pub backspin: Option<Backspin>,
    pub doubleupper: Option<Doubleupper>,
}

#[derive(Debug)]
pub struct Shoot2 {
    pub extra: BTreeMap<i64, Extra>,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct OverSwingDouble {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct TripleBlow {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct SwingO1 {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Action {
    pub extra: BTreeMap<i64, Extra>,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct SwingP2 {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct QuadBlow {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct BackEffect0 {
    pub z: i64,
}

#[derive(Debug)]
pub struct Summon {
    pub fly: Option<BTreeMap<i64, Fly>>,
    pub attack_1: Option<Attack1>,
    pub summoned: Option<BTreeMap<i64, Summoned>>,
    pub die: Option<BTreeMap<i64, Die>>,
    pub height: Option<i64>,
    pub effect_0: Option<Effect0>,
    pub repeat_0: Option<Repeat0>,
    pub attack_triangle: Option<AttackTriangle>,
    pub stand: Option<BTreeMap<i64, Stand>>,
    pub hit: Option<BTreeMap<i64, Hit>>,
    pub attack_2: Option<Attack2>,
    pub effect: Option<Effect>,
    pub prepare: Option<BTreeMap<i64, Prepare>>,
    pub repeat: Option<Repeat>,
    pub heal: Option<Heal>,
}

#[derive(Debug)]
pub struct Stand {
    pub info: Info,
}

#[derive(Debug)]
pub struct Data {
    pub z: Option<i64>,
    pub effect_0: Option<BTreeMap<i64, Effect0>>,
    pub pdd: Option<i64>,
    pub acc: Option<i64>,
    pub dot: Option<String>,
    pub data: Option<BTreeMap<i64, Data>>,
    pub hit_after: Option<i64>,
    pub speed: Option<i64>,
    pub special: Option<BTreeMap<i64, Special>>,
    pub effect_1: Option<BTreeMap<i64, Effect1>>,
    pub cooltime: Option<i64>,
    pub date_expire: Option<String>,
    pub afterimage: Option<Afterimage>,
    pub damagepc: Option<i64>,
    pub attachfacing: Option<i64>,
    pub hs: Option<String>,
    pub criticaldamage_max: Option<i64>,
    pub item_con: Option<i64>,
    pub y: Option<i64>,
    pub damage: Option<String>,
    pub extra: Option<BTreeMap<i64, Extra>>,
    pub attack_count: Option<i64>,
    pub eva: Option<i64>,
    pub rb: Option<Vec2>,
    pub pos: Option<i64>,
    pub dot_time: Option<String>,
    pub dot_interval: Option<String>,
    pub ball: Option<BTreeMap<i64, Ball>>,
    pub mad: Option<i64>,
    pub only_once: Option<i64>,
    pub effect: Option<BTreeMap<i64, Effect>>,
    pub x: Option<i64>,
    pub mastery: Option<i64>,
    pub mdd: Option<i64>,
    pub lt: Option<Vec2>,
    pub mp_con: Option<i64>,
    pub mob_count: Option<i64>,
    pub item_con_no: Option<i64>,
    pub prop: Option<i64>,
    pub jump: Option<i64>,
    pub hit: Option<Hit>,
    pub range: Option<i64>,
    pub fixdamage: Option<i64>,
    pub time: Option<i64>,
    pub pad: Option<i64>,
}

#[derive(Debug)]
pub struct Extra {
    pub skill_lv: Option<i64>,
    pub rb: Option<Vec2>,
    pub count: Option<i64>,
    pub fall: Option<i64>,
    pub duration: Option<i64>,
    pub y: Option<i64>,
    pub x: Option<i64>,
    pub effect_distance: Option<i64>,
    pub interval: Option<i64>,
    pub order: Option<i64>,
    pub start: Option<i64>,
    pub lt: Option<Vec2>,
}

#[derive(Debug)]
pub struct Mob {
    pub repeat: Option<i64>,
    pub icon: Option<i64>,
    pub pos: i64,
}

#[derive(Debug)]
pub struct DeathBlow {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct StabO2 {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Keydown {
    pub action: String,
    pub time: i64,
}

#[derive(Debug)]
pub struct SpecialAction {
    pub extra: BTreeMap<i64, Extra>,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Axe {
    pub stab_o_2: StabO2,
}

#[derive(Debug)]
pub struct SwordOs {
    pub stab_o_1: Option<StabO1>,
    pub stab_o_2: StabO2,
    pub swing_o_2: Option<SwingO2>,
    pub swing_o_1: Option<SwingO1>,
    pub swing_o_3: Option<SwingO3>,
}

#[derive(Debug)]
pub struct ProneStab {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Effect0 {
    pub z: i64,
}

#[derive(Debug)]
pub struct Ball {
    pub extra: Option<BTreeMap<i64, Extra>>,
    pub delay: Option<i64>,
    pub rotate_period: Option<i64>,
    pub data: Option<BTreeMap<i64, Data>>,
}

#[derive(Debug)]
pub struct CharLevel {
    pub data: BTreeMap<i64, Data>,
    pub extra: BTreeMap<i64, Extra>,
}

#[derive(Debug)]
pub struct Attack2 {
    pub info: Info,
}

#[derive(Debug)]
pub struct Hit {
    pub data: BTreeMap<i64, Data>,
    pub extra: BTreeMap<i64, Extra>,
}

#[derive(Debug)]
pub struct Shot {
    pub extra: BTreeMap<i64, Extra>,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Heal {
    pub info: Info,
}

#[derive(Debug)]
pub struct Mace {
    pub stab_o_2: Option<StabO2>,
    pub death_blow: Option<DeathBlow>,
    pub quad_blow: Option<QuadBlow>,
    pub finish_blow: Option<FinishBlow>,
    pub finish_attack_link: Option<FinishAttackLink>,
    pub triple_blow: Option<TripleBlow>,
}

#[derive(Debug)]
pub struct Req {
    pub extra: BTreeMap<i64, Extra>,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Prepare {
    pub time: Option<i64>,
    pub action: String,
    pub z: Option<i64>,
}

#[derive(Debug)]
pub struct SwingT3 {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Keydownend {
    pub action: String,
    pub time: i64,
}

#[derive(Debug)]
pub struct Finish {
    pub extra: BTreeMap<i64, Extra>,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Effect {
    pub extra: Option<BTreeMap<i64, Extra>>,
    pub z: Option<i64>,
    pub pos: Option<i64>,
    pub data: Option<BTreeMap<i64, Data>>,
    pub action: Option<String>,
}

#[derive(Debug)]
pub struct FinishAttackLink {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Spear {
    pub swing_p_2: SwingP2,
    pub swing_p_1: SwingP1,
}

#[derive(Debug)]
pub struct Hit1 {
    pub data: BTreeMap<i64, Data>,
    pub extra: BTreeMap<i64, Extra>,
}

#[derive(Debug)]
pub struct SwingO2 {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct SwingO3 {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct Keydown0 {
    pub z: i64,
}

#[derive(Debug)]
pub struct Repeat0 {
    pub z: i64,
}

#[derive(Debug)]
pub struct Repeat {
    pub flip: Option<i64>,
    pub z: i64,
    pub pos: Option<i64>,
}

#[derive(Debug)]
pub struct SwingT1 {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Bow {
    pub swing_t_3: SwingT3,
    pub prone_stab: Option<ProneStab>,
    pub swing_t_1: SwingT1,
}

#[derive(Debug)]
pub struct SwingP1 {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct Special {
    pub repeat: Option<i64>,
    pub swing_t_1: Option<BTreeMap<i64, SwingT1>>,
    pub shoot_2: Option<BTreeMap<i64, Shoot2>>,
    pub z: Option<i64>,
    pub swing_p_2: Option<BTreeMap<i64, SwingP2>>,
    pub stab_o_2: Option<BTreeMap<i64, StabO2>>,
    pub swing_o_2: Option<BTreeMap<i64, SwingO2>>,
    pub swing_t_3: Option<BTreeMap<i64, SwingT3>>,
    pub heal: Option<BTreeMap<i64, Heal>>,
    pub stab_t_1: Option<BTreeMap<i64, StabT1>>,
    pub extra: Option<Extra>,
    pub swing_o_3: Option<BTreeMap<i64, SwingO3>>,
    pub pos: Option<i64>,
    pub swing_o_1: Option<BTreeMap<i64, SwingO1>>,
    pub stab_o_1: Option<BTreeMap<i64, StabO1>>,
    pub swing_p_1: Option<BTreeMap<i64, SwingP1>>,
    pub prone_stab: Option<BTreeMap<i64, ProneStab>>,
    pub data: Option<BTreeMap<i64, Data>>,
    pub fly: Option<BTreeMap<i64, Fly>>,
}

#[derive(Debug)]
pub struct Attack1 {
    pub info: Info,
}

#[derive(Debug)]
pub struct Info {
    pub attack_after: Option<i64>,
    pub mob_count: Option<i64>,
    pub ball: Option<BTreeMap<i64, Ball>>,
    pub bullet_speed: Option<i64>,
    pub attack_count: Option<String>,
    pub effect: Option<BTreeMap<i64, Effect>>,
    //pub type: Option<i64>,
    pub range: Option<Range>,
    pub priority: Option<i64>,
    pub max_level: Option<i64>,
    pub hit: Option<BTreeMap<i64, Hit>>,
    pub effect_after: Option<i64>,
}

#[derive(Debug)]
pub struct Special0 {
    pub z: i64,
}

#[derive(Debug)]
pub struct FinishBlow {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct Tile {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct SwordTs {
    pub stab_o_2: StabO2,
}

#[derive(Debug)]
pub struct AttackTriangle {
    pub info: Info,
}

#[derive(Debug)]
pub struct Common {
    pub lt: Option<Vec2>,
    pub y: Option<String>,
    pub mastery: Option<String>,
    pub emdd: Option<String>,
    pub epdd: Option<String>,
    pub er: Option<String>,
    pub ignore_mobpdp_r: Option<String>,
    pub z: Option<String>,
    pub range: Option<String>,
    pub bullet_count: Option<String>,
    pub dot_time: Option<String>,
    pub pad_x: Option<String>,
    pub item_con: Option<String>,
    pub mhp_r: Option<String>,
    pub hp_con: Option<String>,
    pub attack_count: Option<String>,
    pub asr_r: Option<String>,
    pub dot: Option<String>,
    pub hp: Option<String>,
    pub jump: Option<String>,
    pub item_con_no: Option<String>,
    pub sub_time: Option<String>,
    pub morph: Option<String>,
    pub dot_interval: Option<String>,
    pub u: Option<String>,
    pub x: Option<String>,
    pub emhp: Option<String>,
    pub money_con: Option<String>,
    pub item_consume: Option<String>,
    pub cooltime: Option<String>,
    pub pdd_r: Option<String>,
    pub cr: Option<String>,
    pub self_destruction: Option<String>,
    pub mad_x: Option<String>,
    pub rb: Option<Vec2>,
    pub mp: Option<String>,
    pub time: Option<String>,
    pub mob_count: Option<String>,
    pub mdd: Option<String>,
    pub meso_r: Option<String>,
    pub damage: Option<String>,
    pub max_level: i64,
    pub emmp: Option<String>,
    pub ter_r: Option<String>,
    pub eva: Option<String>,
    pub epad: Option<String>,
    pub t: Option<String>,
    pub mp_con: Option<String>,
    pub dam_r: Option<String>,
    pub pdd: Option<String>,
    pub criticaldamage_max: Option<String>,
    pub acc: Option<String>,
    pub action: Option<String>,
    pub sub_prop: Option<String>,
    pub criticaldamage_min: Option<String>,
    pub pad: Option<String>,
    pub v: Option<String>,
    pub speed: Option<String>,
    pub bullet_consume: Option<String>,
    pub exp_r: Option<String>,
    pub mmp_r: Option<String>,
    pub mdd_r: Option<String>,
    pub w: Option<String>,
    pub mad: Option<String>,
    pub prop: Option<String>,
}

#[derive(Debug)]
pub struct Die {
    pub info: Info,
}

#[derive(Debug)]
pub struct SwordOl {
    pub swing_o_1: Option<SwingO1>,
    pub swing_o_3: Option<SwingO3>,
    pub stab_o_2: StabO2,
    pub swing_o_2: Option<SwingO2>,
    pub stab_o_1: Option<StabO1>,
}

#[derive(Debug)]
pub struct Gun {
    pub shot: Shot,
}

#[derive(Debug)]
pub struct Finish0 {
    pub z: i64,
}

#[derive(Debug)]
pub struct Barehands {
    pub fist: Option<Fist>,
    pub backspin: Option<Backspin>,
    pub doubleupper: Option<Doubleupper>,
}

#[derive(Debug)]
pub struct FullSwingTriple {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct Hit0 {
    pub data: BTreeMap<i64, Data>,
    pub extra: BTreeMap<i64, Extra>,
}

#[derive(Debug)]
pub struct Fist {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct OverSwingTriple {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct StabT1 {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

#[derive(Debug)]
pub struct SpecialActionFrame {
    pub delay: i64,
}

#[derive(Debug)]
pub struct PoleArm {
    pub swing_p_2: Option<SwingP2>,
    pub full_swing_triple: Option<FullSwingTriple>,
    pub swing_p_1: Option<SwingP1>,
    pub over_swing_triple: Option<OverSwingTriple>,
    pub full_swing_double: Option<FullSwingDouble>,
    pub over_swing_double: Option<OverSwingDouble>,
}

#[derive(Debug)]
pub struct Mob0 {
    pub repeat: Option<i64>,
    pub fixed: Option<i64>,
    pub pos: i64,
}

#[derive(Debug)]
pub struct CrossBow {
    pub shoot_2: Option<Shoot2>,
    pub swing_t_1: Option<SwingT1>,
    pub stab_t_1: Option<StabT1>,
    pub prone_stab: Option<ProneStab>,
}

#[derive(Debug)]
pub struct Effect1 {
    pub z: i64,
}

#[derive(Debug)]
pub struct Range {
    pub lt: Option<Vec2>,
    pub rb: Option<Vec2>,
    pub sp: Option<Vec2>,
    pub r: Option<i64>,
}

#[derive(Debug)]
pub struct Skill {
    pub finish_0: Option<Finish0>,
    pub back_effect_0: Option<BackEffect0>,
    pub hit: Option<Hit>,
    pub time_limited: Option<i64>,
    pub hit_1: Option<Hit1>,
    pub keydown: Option<BTreeMap<i64, Keydown>>,
    pub tile: Option<Tile>,
    pub weapon: Option<i64>,
    pub elem_attr: Option<String>,
    pub mob_code: Option<i64>,
    pub mob: Option<Mob>,
    pub skill: Option<BTreeMap<i64, Skill>>,
    pub affected: Option<Affected>,
    pub disable: Option<i64>,
    pub keydown_0: Option<Keydown0>,
    pub action: Option<Action>,
    pub info: Option<BTreeMap<i64, Info>>,
    pub req: Option<Req>,
    pub afterimage: Option<Afterimage>,
    pub sub_weapon: Option<i64>,
    pub special_action_frame: Option<SpecialActionFrame>,
    pub skill_type: Option<i64>,
    pub invisible: Option<i64>,
    pub hit_0: Option<Hit0>,
    pub char_level: Option<CharLevel>,
    pub psd: Option<i64>,
    pub repeat: Option<BTreeMap<i64, Repeat>>,
    pub level: Option<Level>,
    pub effect_1: Option<BTreeMap<i64, Effect1>>,
    pub summon: Option<Summon>,
    pub common: Option<Common>,
    pub final_attack: Option<FinalAttack>,
    pub effect: Option<BTreeMap<i64, Effect>>,
    pub keydownend: Option<BTreeMap<i64, Keydownend>>,
    pub effect_0: Option<BTreeMap<i64, Effect0>>,
    pub master_level: Option<i64>,
    pub psd_skill: Option<PsdSkill>,
    pub special_0: Option<Special0>,
    pub combat_orders: Option<i64>,
    pub finish: Option<Finish>,
    pub mob_0: Option<Mob0>,
    pub ball: Option<BTreeMap<i64, Ball>>,
    pub special_action: Option<SpecialAction>,
    pub special: Option<Special>,
    pub prepare: Option<Prepare>,
}

#[derive(Debug)]
pub struct FinalAttack {
    pub data: BTreeMap<i64, Data>,
    pub extra: BTreeMap<i64, Extra>,
}

#[derive(Debug)]
pub struct Backspin {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct Afterimage {
    pub sword_tl: Option<SwordTl>,
    pub pole_arm: Option<PoleArm>,
    pub sword_ts: Option<SwordTs>,
    pub bow: Option<Bow>,
    pub spear: Option<Spear>,
    pub axe: Option<Axe>,
    pub gun: Option<Gun>,
    pub mace: Option<Mace>,
    pub cross_bow: Option<CrossBow>,
    pub sword_ol: Option<SwordOl>,
    pub sword_os: Option<SwordOs>,
    pub barehands: Option<Barehands>,
    pub knuckle: Option<Knuckle>,
}

#[derive(Debug)]
pub struct Level {
    pub data: BTreeMap<i64, Data>,
    pub extra: BTreeMap<i64, Extra>,
}

#[derive(Debug)]
pub struct Affected {
    pub pos: Option<i64>,
    pub z: Option<i64>,
    pub data: Option<BTreeMap<i64, Data>>,
    pub extra: Option<BTreeMap<i64, Extra>>,
    pub repeat: Option<i64>,
}

#[derive(Debug)]
pub struct StabO1 {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct FullSwingDouble {
    pub extra: Extra,
    pub data: BTreeMap<i64, Data>,
}

#[derive(Debug)]
pub struct SwordTl {
    pub stab_o_2: StabO2,
}

#[derive(Debug)]
pub struct Doubleupper {
    pub data: BTreeMap<i64, Data>,
    pub extra: Extra,
}

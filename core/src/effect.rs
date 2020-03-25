use num_traits::cast::{FromPrimitive, ToPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, FromPrimitive, ToPrimitive)]
pub enum StatusEffect {
    Speed,
    Slowness,
    Haste,
    MiningFatigue,
    Strength,
    InstantHealth,
    InstantDamage,
    JumpBoost,
    Nausea,
    Regeneration,
    Resistance,
    FireResistance,
    WaterBreathing,
    Invisibility,
    Blindness,
    NightVision,
    Hunger,
    Weakness,
    Poison,
    Wither,
    HealthBoost,
    Absorption,
    Saturation,
    Glowing,
    Levitation,
    Luck,
    BadLuck,
    SlowFalling,
    ConduitPower,
    DolphinsGrace,
    BadOmen,
    HeroOfTheVillage,
}

impl StatusEffect {
    pub fn protocol_id(self) -> i8 {
        self.to_i8().unwrap() + 1
    }

    pub fn from_protocol_id(id: i8) -> Option<StatusEffect> {
        Self::from_i8(id - 1)
    }

    pub fn identifier(self) -> &'static str {
        match self {
            StatusEffect::Speed => "speed",
            StatusEffect::Slowness => "slowness",
            StatusEffect::Haste => "haste",
            StatusEffect::MiningFatigue => "mining_fatigue",
            StatusEffect::Strength => "strength",
            StatusEffect::InstantHealth => "instant_health",
            StatusEffect::InstantDamage => "instant_damage",
            StatusEffect::JumpBoost => "jump_boost",
            StatusEffect::Nausea => "nausea",
            StatusEffect::Regeneration => "regeneration",
            StatusEffect::Resistance => "resistance",
            StatusEffect::FireResistance => "fire_resistance",
            StatusEffect::WaterBreathing => "water_breathing",
            StatusEffect::Invisibility => "invisibility",
            StatusEffect::Blindness => "blindness",
            StatusEffect::NightVision => "night_vision",
            StatusEffect::Hunger => "hunger",
            StatusEffect::Weakness => "weakness",
            StatusEffect::Poison => "poison",
            StatusEffect::Wither => "wither",
            StatusEffect::HealthBoost => "health_boost",
            StatusEffect::Absorption => "absorption",
            StatusEffect::Saturation => "saturation",
            StatusEffect::Glowing => "glowing",
            StatusEffect::Levitation => "levitation",
            StatusEffect::Luck => "luck",
            StatusEffect::BadLuck => "unluck",
            StatusEffect::SlowFalling => "slow_falling",
            StatusEffect::ConduitPower => "conduit_power",
            StatusEffect::DolphinsGrace => "dolphins_grace",
            StatusEffect::BadOmen => "bad_omen",
            StatusEffect::HeroOfTheVillage => "hero_of_the_village",
        }
    }
}

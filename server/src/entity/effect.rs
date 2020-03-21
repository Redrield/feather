use bitflags::bitflags;

bitflags! {
    pub struct EffectFlags: u8 {
        const AMBIENT = 0x01;
        const SHOW_PARTICLES = 0x02;
        const SHOW_ICON = 0x04;
    }
}

macro_rules! gen_effect_components {
    ($($effect:ident),+) => {
    $(
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub struct $effect {
            entity_id: $crate::entity::EntityId,
            amplifier: u8,
            time_remaining: i32,
            flags: EffectFlags,
        }

        impl $effect {
            pub fn get_effect(self) -> $crate::feather_core::StatusEffect {
                $crate::feather_core::StatusEffect::$effect
            }
        }
    )+
    }
}

gen_effect_components!(
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
    HeroOfTheVillage
);
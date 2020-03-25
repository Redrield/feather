use std::collections::btree_map::BTreeMap;
use uuid::Uuid;
use crate::network::mctypes::{McTypeWrite, McTypeRead};
use crate::bytes_ext::{BytesMutExt, BytesExt};
use bytes::Buf;

#[derive(Default, Clone)]
pub struct EntityProperties {
    pub props: BTreeMap<String, EntityProperty>
}

impl EntityProperties {
    pub fn new() -> EntityProperties {
        let mut props = BTreeMap::new();
        // Values from wiki.vg for 1.13.2
        props.insert("generic.maxHealth".to_string(), EntityProperty::new(20.0));
        props.insert("generic.followRange".to_string(), EntityProperty::new(32.0));
        props.insert("generic.knockbackResistance".to_string(), EntityProperty::new(0.0));
        props.insert("generic.movementSpeed".to_string(), EntityProperty::new(0.699999988079071));
        props.insert("generic.attackDamage".to_string(), EntityProperty::new(2.0));
        props.insert("generic.attackSpeed".to_string(), EntityProperty::new(4.0));
        props.insert("generic.flyingSpeed".to_string(), EntityProperty::new(0.4000000059604645));

        Self { props }
    }

    pub fn add_property(&mut self, key: String, property: EntityProperty) {
        self.props.insert(key, property);
    }

    pub fn get_property(&self, key: &str) -> Option<&EntityProperty> {
        self.props.get(key)
    }

    pub fn get_property_mut(&mut self, key: &str) -> Option<&mut EntityProperty> {
        self.props.get_mut(key)
    }
}

#[derive(Default, Clone)]
pub struct EntityProperty {
    base_value: f64,
    modifiers: Vec<PropertyModifier>,
}

impl EntityProperty {
    pub fn new(base_value: f64) -> EntityProperty {
        EntityProperty {
            base_value,
            modifiers: Vec::new()
        }
    }

    pub fn add_modifier(&mut self, modifier: PropertyModifier) {
        self.modifiers.push(modifier);
    }
}

#[derive(Default, Clone)]
pub struct PropertyModifier {
    uuid: Uuid,
    amount: f64,
    operation: ModifierOperation
}

impl PropertyModifier {
    pub fn new(uuid: Uuid, amount: f64, operation: ModifierOperation) -> PropertyModifier {
        PropertyModifier { uuid, amount, operation }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ModifierOperation {
    Add,
    AddPercent,
    Multiply
}

impl Default for ModifierOperation {
    fn default() -> Self {
        ModifierOperation::Add
    }
}

impl ModifierOperation {
    pub fn to_protocol(self) -> i8 {
        match self {
            ModifierOperation::Add => 0,
            ModifierOperation::AddPercent => 1,
            ModifierOperation::Multiply => 2,
        }
    }

    pub fn from_protocol(proto_value: i8) -> Option<Self> {
        match proto_value {
            0 => Some(ModifierOperation::Add),
            1 => Some(ModifierOperation::AddPercent),
            2 => Some(ModifierOperation::Multiply),
            _ => None
        }
    }
}

pub trait EntityPropertyWrite {
    fn push_properties(&mut self, properties: &EntityProperties);

    fn push_property(&mut self, property: &EntityProperty);

    fn push_modifier(&mut self, modifier: &PropertyModifier);
}

pub trait EntityPropertyRead {
    fn try_get_properties(&mut self) -> anyhow::Result<EntityProperties>;

    fn try_get_property(&mut self) -> anyhow::Result<EntityProperty>;

    fn try_get_modifier(&mut self) -> anyhow::Result<PropertyModifier>;
}

impl<B> EntityPropertyWrite for B
    where
        B: BytesMutExt + McTypeWrite,
{
    fn push_properties(&mut self, properties: &EntityProperties) {
        self.push_i32(properties.props.len() as i32);

        for (key, property) in properties.props.iter() {
            self.push_string(key);
            self.push_property(property);
        }
    }

    fn push_property(&mut self, property: &EntityProperty) {
        self.push_f64(property.base_value);
        self.push_var_int(property.modifiers.len() as i32);

        for modifier in &property.modifiers {
            self.push_modifier(modifier);
        }
    }

    fn push_modifier(&mut self, modifier: &PropertyModifier) {
        self.push_uuid(&modifier.uuid);
        self.push_f64(modifier.amount);
        self.push_i8(modifier.operation.to_protocol());
    }
}

impl<B> EntityPropertyRead for B
where
    B: Buf + std::io::Read
{
    fn try_get_properties(&mut self) -> anyhow::Result<EntityProperties> {
        let num_props = self.try_get_i32()?;

        let mut props = BTreeMap::new();

        for _ in 0..num_props {
            let key = self.try_get_string()?;
            let property = self.try_get_property()?;
            props.insert(key, property);
        }

        Ok(EntityProperties { props })
    }

    fn try_get_property(&mut self) -> anyhow::Result<EntityProperty> {
        let value = self.try_get_f64()?;
        let num_modifiers = self.try_get_var_int()?;

        let mut modifiers = Vec::new();

        for _ in 0..num_modifiers {
            modifiers.push(self.try_get_modifier()?);
        }

        Ok(EntityProperty { base_value: value, modifiers })
    }

    fn try_get_modifier(&mut self) -> anyhow::Result<PropertyModifier> {
        let uuid = self.try_get_uuid()?;
        let amount = self.try_get_f64()?;
        let operation = ModifierOperation::from_protocol(self.try_get_i8()?).unwrap();

        Ok(PropertyModifier { uuid, amount, operation })
    }
}
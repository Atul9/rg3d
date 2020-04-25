use crate::{
    core::{
        color::Color,
        visitor::{Visit, VisitResult, Visitor},
    },
    resource::texture::Texture,
    scene::base::{Base, BaseBuilder},
};
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Sprite {
    base: Base,
    texture: Option<Arc<Mutex<Texture>>>,
    color: Color,
    size: f32,
    rotation: f32,
}

impl Deref for Sprite {
    type Target = Base;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for Sprite {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for Sprite {
    fn default() -> Self {
        SpriteBuilder::new(BaseBuilder::new()).build()
    }
}

impl Sprite {
    pub fn set_size(&mut self, size: f32) {
        self.size = size;
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn color(&self) -> Color {
        self.color
    }

    /// Sets rotation around "look" axis in radians.
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_texture(&mut self, texture: Arc<Mutex<Texture>>) {
        self.texture = Some(texture);
    }

    pub fn texture(&self) -> Option<Arc<Mutex<Texture>>> {
        self.texture.clone()
    }
}

impl Visit for Sprite {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        visitor.enter_region(name)?;

        self.texture.visit("Texture", visitor)?;
        self.color.visit("Color", visitor)?;
        self.size.visit("Size", visitor)?;
        self.rotation.visit("Rotation", visitor)?;
        self.base.visit("Base", visitor)?;

        visitor.leave_region()
    }
}

pub struct SpriteBuilder {
    base_builder: BaseBuilder,
    texture: Option<Arc<Mutex<Texture>>>,
    color: Option<Color>,
    size: Option<f32>,
    rotation: Option<f32>,
}

impl SpriteBuilder {
    pub fn new(base_builder: BaseBuilder) -> Self {
        Self {
            base_builder,
            texture: None,
            color: None,
            size: None,
            rotation: None,
        }
    }

    pub fn with_texture(mut self, texture: Arc<Mutex<Texture>>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_opt_texture(mut self, texture: Option<Arc<Mutex<Texture>>>) -> Self {
        self.texture = texture;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = Some(rotation);
        self
    }

    pub fn build(self) -> Sprite {
        Sprite {
            base: self.base_builder.build(),
            texture: self.texture,
            color: self.color.unwrap_or(Color::WHITE),
            size: self.size.unwrap_or(0.2),
            rotation: self.rotation.unwrap_or(0.0),
        }
    }
}

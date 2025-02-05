use macroquad::{
    experimental::{
        animation::{
            AnimatedSprite,
            Animation,
        },
    },
    color,
    prelude::*,
};

use serde::{
    Serialize,
    Deserialize,
};

use crate::{
    Resources,
    get_global,
    json,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct SpriteAnimationParams {
    pub offset: json::Vec2,
    pub texture_id: String,
    pub tile_size: json::Vec2,
    pub animations: Vec<json::Animation>,
    pub should_play: Option<bool>,
}

impl Default for SpriteAnimationParams {
    fn default() -> Self {
        SpriteAnimationParams {
            offset: json::Vec2::new(-8.0, -8.0),
            texture_id: Resources::WHITE_TEXTURE_ID.to_string(),
            tile_size: json::Vec2::new(16.0, 16.0),
            animations: vec!(
                json::Animation {
                    name: "idle".to_string(),
                    row: 0,
                    frames: 1,
                    fps: 8
                },
            ),
            should_play: Some(false),
        }
    }
}

#[derive(Clone)]
pub struct SpriteAnimationPlayer {
    pub offset: Vec2,
    pub rotation: f32,
    pub flip_x: bool,
    pub flip_y: bool,
    texture_id: String,
    tile_size: Vec2,
    animations: Vec<Animation>,
    animated_sprite: AnimatedSprite,
}

impl SpriteAnimationPlayer {
    pub fn new(params: SpriteAnimationParams) -> Self {
        let animations: Vec<Animation> = params.animations.iter().map(|anim| anim.to_macroquad()).collect();
        let sprite = AnimatedSprite::new(
            params.tile_size.x as u32,
            params.tile_size.y as u32,
            &animations,
            params.should_play.unwrap_or_default(),
        );

        SpriteAnimationPlayer {
            offset: params.offset.to_macroquad(),
            rotation: 0.0,
            flip_x: false,
            flip_y: false,
            texture_id: params.texture_id.to_string(),
            tile_size: params.tile_size.to_macroquad(),
            animations,
            animated_sprite: sprite,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.animated_sprite.playing
    }

    pub fn set_animation(&mut self, id: usize) {
        self.animated_sprite.set_animation(id);
    }

    pub fn start_animation(&mut self, id: usize) {
        self.set_animation(id);
        self.play();
    }

    pub fn restart_animation(&mut self) {
        self.animated_sprite.set_frame(0);
    }

    pub fn set_frame(&mut self, frame: u32) {
        self.animated_sprite.set_frame(frame);
    }

    pub fn play(&mut self) {
        self.animated_sprite.playing = true;
    }

    pub fn stop(&mut self) {
        self.animated_sprite.playing = false;
    }

    pub fn to_sprite_params(&self) -> SpriteAnimationParams {
        SpriteAnimationParams {
            offset: json::Vec2::from(self.offset),
            texture_id: self.texture_id.to_string(),
            tile_size: json::Vec2::from(self.tile_size),
            animations: self.animations.iter().map(|anim| json::Animation::from(anim.clone())).collect(),
            should_play: Some(self.animated_sprite.playing),
        }
    }

    pub fn update(&mut self) {
        self.animated_sprite.update();
    }

    pub fn draw(&mut self, position: Vec2, rotation: f32) {
        let resources = get_global::<Resources>();
        draw_texture_ex(
            resources.get_texture(&self.texture_id).clone(),
            position.x + self.offset.x,
            position.y + self.offset.y,
            color::WHITE,
            DrawTextureParams {
                source: Some(self.animated_sprite.frame().source_rect),
                dest_size: Some(self.animated_sprite.frame().dest_size),
                flip_x: self.flip_x,
                flip_y: self.flip_y,
                rotation: self.rotation + rotation,
                ..Default::default()
            },
        );
    }
}

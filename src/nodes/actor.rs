use std::ops::Sub;

use serde::{
    Serialize,
    Deserialize,
};

use macroquad::{
    experimental::{
        scene::{
            Node,
            Handle,
            RefMut,
        },
    },
    color,
    prelude::*,
};

use crate::input::apply_local_player_input;

mod controller;
mod inventory;
mod ability;
mod draw_buffer;
mod stats;

pub use stats::ActorStats;
pub use controller::{
    ActorControllerKind,
    ActorController,
};

pub use inventory::ActorInventory;
pub use ability::{
    ActorAbilityParams,
    ActorAbility,
};

pub use draw_buffer::ActorDrawBuffer;

use crate::{get_global, render::{
    SpriteAnimationPlayer,
    SpriteAnimationParams,
    draw_progress_bar,
    HorizontalAlignment,
}, globals::LocalPlayer, physics::{
    PhysicsBody,
    PhysicsObject,
    Collider,
}, json, generate_id, draw_aligned_text};
use crate::nodes::Item;
use crate::render::Viewport;

#[derive(Clone, Serialize, Deserialize)]
pub struct ActorParams {
    pub id: String,
    pub position: Option<json::Vec2>,
    pub name: String,
    pub stats: json::ActorStats,
    pub factions: Vec<String>,
    pub collider: Option<json::Collider>,
    pub inventory: Vec<String>,
    pub sprite_animation_params: SpriteAnimationParams,
}

impl Default for ActorParams {
    fn default() -> Self {
        ActorParams {
            id: generate_id(),
            position: None,
            name: "Unnamed Actor".to_string(),
            stats: Default::default(),
            factions: Vec::new(),
            collider: None,
            inventory: Vec::new(),
            sprite_animation_params: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct Actor {
    pub id: String,
    pub name: String,
    pub stats: ActorStats,
    pub factions: Vec<String>,
    pub body: PhysicsBody,
    sprite_animation: SpriteAnimationPlayer,
    pub inventory: ActorInventory,
    pub primary_ability: Option<ActorAbility>,
    pub secondary_ability: Option<ActorAbility>,
    pub controller: ActorController,
}

impl Actor {
    const HEALTH_BAR_LENGTH: f32 = 50.0;
    const HEALTH_BAR_HEIGHT: f32 = 10.0;
    const HEALTH_BAR_OFFSET_Y: f32 = 25.0;

    const ENCUMBERED_SPEED_FACTOR: f32 = 0.1;

    const SPRINT_SPEED_FACTOR: f32 = 2.0;
    const SPRINT_STAMINA_COST: f32 = 10.0;

    const PICK_UP_RADIUS: f32 = 36.0;
    const INTERACT_RADIUS: f32 = 36.0;

    pub fn new(position: Vec2, controller_kind: ActorControllerKind, max_vitals: bool, params: ActorParams) -> Self {
        let collider = match params.collider {
            Some(collider) => Some(collider.to_collider()),
            None => None,
        };
        let body = PhysicsBody::new(position, 0.0, collider);
        Actor {
            id: generate_id(),
            name: params.name,
            stats: params.stats.to_actor_stats(max_vitals),
            factions: params.factions,
            body,
            sprite_animation: SpriteAnimationPlayer::new(params.sprite_animation_params.clone()),
            inventory: ActorInventory::new(&params.inventory),
            primary_ability: None,
            secondary_ability: None,
            controller: ActorController::new(controller_kind),
        }
    }

    pub fn add_node(position: Vec2, controller_kind: ActorControllerKind, max_vitals: bool, params: ActorParams) -> Handle<Self> {
        scene::add_node(Self::new(position, controller_kind, max_vitals, params))
    }

    pub fn to_actor_params(&self) -> ActorParams {
        let collider = match self.body.collider {
            Some(collider) => Some(json::Collider::from(collider)),
            None => None,
        };
        ActorParams {
            id: self.id.clone(),
            position: Some(json::Vec2::from(self.body.position)),
            name: self.name.clone(),
            stats: json::ActorStats::from(self.stats.clone()),
            factions: self.factions.clone(),
            collider,
            inventory: self.inventory.to_item_ids(),
            sprite_animation_params: self.sprite_animation.to_sprite_params(),
        }
    }

    pub fn take_damage(&mut self, _actor_id: &str, damage: f32) {
        self.stats.current_health -= damage;
    }

    pub fn find_player(player_id: u32) -> Option<RefMut<Self>> {
        for actor in scene::find_nodes_by_type::<Self>() {
            match actor.controller.kind {
                ActorControllerKind::Player { id } => {
                    if player_id == id {
                        return Some(actor);
                    }
                }
                _ => {}
            }
        }
        None
    }

    pub fn find_local_player() -> Option<RefMut<Self>> {
        let local_player = get_global::<LocalPlayer>();
        if let Some(actor) = Self::find_player(local_player.id) {
            Some(actor)
        } else {
            None
        }
    }

    pub fn find_with_id(id: &str) -> Option<RefMut<Actor>> {
        for actor in scene::find_nodes_by_type::<Actor>() {
            if actor.id == id.to_string() {
                return Some(actor);
            }
        }
        None
    }

    pub fn set_animation(&mut self, direction: Vec2, is_stationary: bool) {
        if direction.x > 0.0 && direction.x.abs() > direction.y.abs() {
            self.sprite_animation.start_animation(2);
            self.sprite_animation.flip_x = false;
        } else if direction.x < 0.0 {
            self.sprite_animation.start_animation(2);
            self.sprite_animation.flip_x = true;
        }else if direction.y > 0.0 && direction.y.abs() > direction.x.abs() {
            self.sprite_animation.start_animation(0);
        } else if direction.y < 0.0 {
            self.sprite_animation.start_animation(1);
        } else {
            self.sprite_animation.set_frame(1);
            self.sprite_animation.stop();
        }
        if is_stationary {
            self.sprite_animation.set_frame(1);
            self.sprite_animation.stop();
        }
    }

    pub fn draw_actor(&mut self) {
        {
            self.body.debug_draw();
            let (position, rotation) = (self.body.position, self.body.rotation);
            self.sprite_animation.draw(position, rotation);
        }

        let is_local_player = self.is_local_player();
        let (position, offset_y, alignment, length, height, border) = if is_local_player {
            let viewport = get_global::<Viewport>();
            let height = Self::HEALTH_BAR_HEIGHT * viewport.s;
            (vec2(10.0, 10.0), height / 2.0, HorizontalAlignment::Left, Self::HEALTH_BAR_LENGTH * viewport.s, height, viewport.s)
        } else {
            (self.body.position, Self::HEALTH_BAR_OFFSET_Y, HorizontalAlignment::Center, Self::HEALTH_BAR_LENGTH, Self::HEALTH_BAR_HEIGHT, 1.0)
        };
        if is_local_player || self.stats.current_health < self.stats.max_health {
            if is_local_player {
                push_camera_state();
                set_default_camera();
            }
            draw_progress_bar(
                self.stats.current_health,
                self.stats.max_health,
                position + vec2(0.0, offset_y),
                length,
                height,
                color::RED,
                color::GRAY,
                border,
                alignment.clone(),
                None, // Some(&format!("{}/{}", self.stats.current_health.round(), self.stats.max_health.round())),
                None,
            );
        }
        if is_local_player {
            draw_aligned_text(
                &format!("position: {}", self.body.position.to_string()) ,
                screen_width() - 50.0,
                50.0,
                HorizontalAlignment::Right,
                Default::default(),
            );
            draw_progress_bar(
                self.stats.current_stamina,
                self.stats.max_stamina,
                position + vec2(0.0, offset_y + height),
                length,
                height,
                color::YELLOW,
                color::GRAY,
                border,
                alignment.clone(),
                None, // Some(&format!("{}/{}", self.stats.current_stamina.round(), self.stats.max_stamina.round())),
                None,
            );
            draw_progress_bar(
                self.stats.current_energy,
                self.stats.max_energy,
                position + vec2(0.0, offset_y + height * 2.0),
                length,
                height,
                color::BLUE,
                color::GRAY,
                border,
                alignment,
                None, // Some(&format!("{}/{}", self.stats.current_energy.round(), self.stats.max_energy.round())),
                None,
            );
            pop_camera_state();
        }
    }

    pub fn is_local_player(&self) -> bool {
        if let ActorControllerKind::Player { id } = self.controller.kind {
            let local_player = get_global::<LocalPlayer>();
            id == local_player.id
        } else {
            false
        }
    }

    pub fn interact(&self, other: &mut Actor) {
        println!("INTERACTION between '{}' and '{}'", self.name, other.name);
    }
}

impl Node for Actor {
    fn ready(mut node: RefMut<Self>) {
        node.provides::<PhysicsObject>((
            node.handle().untyped(),
            node.handle().lens(|actor| &mut actor.body),
        ));
    }

    fn update(mut node: RefMut<Self>) {
        node.stats.update_derived(false);
        node.sprite_animation.update();

        if node.stats.current_health <= 0.0 {
            let position = node.body.position;
            node.inventory.drop_all(position);
            node.delete();
            return;
        }

        if let Some(ability) = node.primary_ability.as_mut() {
            ability.update();
        }

        if let Some(ability) = node.secondary_ability.as_mut() {
            ability.update();
        }

        match node.controller.kind {
            ActorControllerKind::Player { id } => {
                let local_player = get_global::<LocalPlayer>();
                if id == local_player.id {
                    apply_local_player_input(&mut node.controller);
                } else {
                    // TODO: Remote player (?)
                }
            }
            ActorControllerKind::Computer => {
                // TODO: Computer controlled
            }
            ActorControllerKind::None => {}
        }

        let controller_direction = node.controller.direction;
        if let Some(target) = node.controller.primary_target {
            let direction = target.sub(node.body.position).normalize_or_zero();
            node.set_animation(direction, controller_direction == Vec2::ZERO);
        } else if let Some(target) = node.controller.secondary_target {
            let direction = target.sub(node.body.position).normalize_or_zero();
            node.set_animation(direction, controller_direction == Vec2::ZERO);
        } else {
            node.set_animation(controller_direction, false);
        }

        let controller = node.controller.clone();
        if let Some(target) = controller.primary_target {
            let mut primary_ability = node.primary_ability.clone();
            let position = node.body.position.clone();
            if let Some(ability) = &mut primary_ability {
                ability.activate(&mut *node, position, target);
            }
            node.primary_ability = primary_ability;
        }
        if let Some(target) = controller.secondary_target {
            let mut secondary_ability = node.secondary_ability.clone();
            let position = node.body.position.clone();
            if let Some(ability) = &mut secondary_ability {
                ability.activate(&mut *node, position, target);
            }
            node.secondary_ability = secondary_ability;
        }
    }

    fn fixed_update(mut node: RefMut<Self>) {
        let direction = node.controller.direction.normalize_or_zero();
        node.body.velocity = direction * if node.inventory.get_total_weight() >= node.stats.carry_capacity {
            node.stats.move_speed * Self::ENCUMBERED_SPEED_FACTOR
        } else if node.controller.is_sprinting && node.stats.current_stamina >= Self::SPRINT_STAMINA_COST {
            if direction != Vec2::ZERO {
                node.stats.current_stamina -= Self::SPRINT_STAMINA_COST;
            }
            node.stats.move_speed * Self::SPRINT_SPEED_FACTOR
        } else {
            node.stats.move_speed
        };

        node.body.integrate();

        if node.controller.is_picking_up_items {
            let collider = Collider::circle(0.0, 0.0, Self::PICK_UP_RADIUS).offset(node.body.position);
            for item in scene::find_nodes_by_type::<Item>() {
                if collider.contains(item.position) {
                    node.inventory.pick_up_item(item);
                }
            }
        }

        if node.controller.is_interacting {
            let collider = Collider::circle(0.0, 0.0, Self::INTERACT_RADIUS).offset(node.body.position);
            for actor in scene::find_nodes_by_type::<Actor>() {
                if let Some(other_collider) = actor.body.get_offset_collider() {
                    if collider.overlaps(&other_collider) {
                        for faction in &node.factions {
                            if actor.factions.contains(faction) {
                                actor.interact(&mut *node);
                                node.controller.is_interacting = false; // stop this form firing twice
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw(node: RefMut<Self>) {
        let mut draw_queue = scene::find_node_by_type::<ActorDrawBuffer>().unwrap();
        draw_queue.add_to_buffer(node.handle());
    }
}

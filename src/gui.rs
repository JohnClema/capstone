use macroquad::{
    ui::{
        hash, root_ui,
        widgets::{self, Group},
        Drag, Ui,
    },
    prelude::*,
};

use crate::{
    nodes::{GameState, Actor},
    Item,
};

pub fn draw_gui() {
    let game_state = scene::find_node_by_type::<GameState>().unwrap();
    let player = Actor::find_local_player().unwrap();
    if game_state.show_character_window {
        widgets::Window::new(hash!(), vec2(50.0, 50.0), vec2(300.0, 300.0))
            .label(&player.name)
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, &format!("STR: {}", player.stats.strength));
                ui.label(None, &format!("DEX: {}", player.stats.dexterity));
                ui.label(None, &format!("CON: {}", player.stats.constitution));
                ui.label(None, &format!("INT: {}", player.stats.intelligence));
                ui.label(None, &format!("WIL: {}", player.stats.willpower));
                ui.label(None, &format!("PER: {}", player.stats.perception));
                ui.label(None, &format!("CHA: {}", player.stats.charisma));

                ui.separator();

                ui.tree_node(hash!(), "Regeneration", |ui| {
                    ui.label(None, &format!("Health:  {}", player.stats.health_regen));
                    ui.label(None, &format!("Stamina: {}", player.stats.stamina_regen));
                    ui.label(None, &format!("Energy:  {}", player.stats.energy_regen));
                });
            });
    }
    if game_state.show_inventory_window {
        widgets::Window::new(hash!(), vec2(50.0, 375.0), vec2(300.0, 300.0))
            .label("Inventory")
            .ui(&mut *root_ui(), |ui| {
                ui.tree_node(hash!(), "Weapons", |ui| {
                    for item in &player.inventory.get_all_of_kinds(&[
                        Item::ONE_HANDED_WEAPON_KIND,
                        Item::TWO_HANDED_WEAPON_KIND,
                    ]) {
                        ui.label(None, &item.name);
                    }
                });
                ui.tree_node(hash!(), "Trinkets", |ui| {
                    for item in &player.inventory.get_all_of_kind(Item::TRINKET_KIND) {
                        ui.label(None, &item.name);
                    }
                });
            });
    }
}
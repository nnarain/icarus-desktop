//
// main.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 12 2023
//

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

fn ui_system(mut ctx: ResMut<EguiContext>) {
    egui::Window::new("Icarus Desktop").show(ctx.ctx_mut(), |ui| {
        ui.label("This is text");
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_system(ui_system)
        .run();
}

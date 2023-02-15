//
// main.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 12 2023
//

use bevy::prelude::*;
use bevy_egui::{
    egui::{
        self,
        plot::{Line, Plot, PlotPoints, Legend}
    },
    EguiContext, EguiPlugin
};
use icarus_desktop::{IcarusPlugin, Sensors};

use itertools::Itertools;


fn ui_system(mut ctx: ResMut<EguiContext>, sensors: Res<Sensors>) {
    egui::Window::new("Sensors").show(ctx.ctx_mut(), |ui| {
        let (pitch, roll, yaw): (Vec<_>, Vec<_>, Vec<_>) = sensors.attitude
                                                            .iter()
                                                            .map(|a| (a.pitch as f64, a.roll as f64, a.yaw as f64))
                                                            .multiunzip();

        let pitch: PlotPoints = pitch.iter().enumerate().map(|(i, &p)| [i as f64, p]).collect();
        let roll: PlotPoints = roll.iter().enumerate().map(|(i, &r)| [i as f64, r]).collect();
        let yaw: PlotPoints = yaw.iter().enumerate().map(|(i, &y)| [i as f64, y]).collect();

        let pitch = Line::new(pitch).name("Pitch");
        let roll = Line::new(roll).name("Roll");
        let yaw = Line::new(yaw).name("Yaw");

        Plot::new("attitude")
            .view_aspect(2.0)
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                plot_ui.line(pitch);
                plot_ui.line(roll);
                plot_ui.line(yaw);
            });

    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
            window: WindowDescriptor{title: "Icarus Desktop".into(), ..Default::default()},
            ..Default::default()
        }))
        .add_plugin(EguiPlugin)
        .add_plugin(IcarusPlugin)
        .add_system(ui_system)
        .run();
}

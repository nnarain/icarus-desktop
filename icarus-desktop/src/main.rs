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
        plot::{Line, Plot, PlotPoints, Legend},
        Slider,
    },
    EguiContext, EguiPlugin
};
use icarus_client::Throttle;
use icarus_desktop::{IcarusPlugin, Sensors, ThrottleControl};

use itertools::Itertools;


fn ui_system(mut ctx: ResMut<EguiContext>, sensors: Res<Sensors>, mut throttle_control: ResMut<ThrottleControl>) {
    // Render a window for the sensor data
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

    // Render a window for throttle control
    egui::Window::new("Controls").show(ctx.ctx_mut(), |ui|{
        let Throttle { mut pitch, mut roll, mut yaw, mut vertical } = throttle_control.last().clone();

        ui.add(Slider::new(&mut pitch, 0..=100).text("Pitch"));
        ui.add(Slider::new(&mut roll, 0..=100).text("Roll"));
        ui.add(Slider::new(&mut yaw, 0..=100).text("Yaw"));
        ui.add(Slider::new(&mut vertical, 0..=100).text("Thrust"));

        let throttle = Throttle::new(pitch, roll, yaw, vertical);

        throttle_control.enqueue(throttle);
    });
}

fn environment_setup(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight::default(),
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..Default::default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 6., 6.).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..Default::default()
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
        .add_startup_system(environment_setup)
        .add_system(ui_system)
        .run();
}

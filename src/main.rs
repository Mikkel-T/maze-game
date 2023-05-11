#![windows_subsystem = "windows"]

pub mod endscreen;
pub mod game;
pub mod maze;
pub mod menu;
pub mod utils;

use bevy::{prelude::*, time::Stopwatch};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Maze game".to_string(),
                        resizable: false,
                        resolution: (utils::vars::WIDTH, utils::vars::HEIGHT).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(utils::colors::BACKGROUND_COLOR))
        .add_state::<utils::vars::GameState>()
        .insert_resource(utils::vars::MazeState {
            stopwatch: Stopwatch::new(),
            size: 0,
            path: None,
            coins: 0,
        })
        .add_plugin(menu::MenuPlugin)
        .add_plugin(game::GamePlugin)
        .add_plugin(endscreen::EndScreenPlugin)
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

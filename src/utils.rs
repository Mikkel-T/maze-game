use bevy::prelude::*;

pub mod vars {
    use super::*;
    use crate::maze::Maze;
    use bevy::time::Stopwatch;

    pub const HEIGHT: f32 = 600.;
    pub const WIDTH: f32 = 1000.;
    pub const PLAYER_SPEED: f32 = 6.;
    pub const MAZE_BORDER_WIDTH: f32 = 3.;

    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    pub enum GameState {
        #[default]
        Menu,
        Game,
        EndGame,
    }

    #[derive(Resource)]
    pub struct MazeState {
        pub stopwatch: Stopwatch,
        pub size: usize,
        pub path: Option<Maze>,
    }
}

pub mod colors {
    use super::*;

    pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
    pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
    pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
    pub const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
    pub const WALL_COLOR: Color = Color::BLACK;
    pub const BACKGROUND_COLOR: Color = Color::CRIMSON;
}

pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

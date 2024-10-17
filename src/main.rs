use bevy::asset::io::memory::Dir;
use bevy::prelude::*;
use rand::prelude::*;

fn main() {
    println!("Game Start!");

    println!("222");
    App::new()
        .add_systems(Startup, (setup).chain())
        .add_systems(Update, (snake_direction_change, food_spawner).chain())
        .add_systems(FixedUpdate, (snake_movement).chain())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Snake".to_string(),
                resolution: (200.0, 200.0).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .run();
    println!("Game End!");
}

const ARENA_WIDTH: i32 = 10;
const ARENA_HEIGHT: i32 = 10;

#[derive(Debug)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

#[derive(Deref, DerefMut)]
struct SnakeMoveTimer(Timer);
impl Default for crate::SnakeMoveTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

#[derive(Component)]
struct Food;
#[derive(Deref, DerefMut)]
struct FoodSpawnTimer(Timer);
impl Default for FoodSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

#[derive(Component)]
struct Collider;



fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    // camera
    commands.spawn(Camera2dBundle::default());

    // snake head
    for window in windows.iter_mut() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(0.7, 0.7, 0.7, 1.0),
                    custom_size: Some(Vec2::new(get_width(window.width()), get_height(window.height()))),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(get_width(window.width()) / 2.0, get_height(window.height()) / 2.0, 0.0),
                    ..default()
                },
                ..default()
            },
            SnakeHead {
                direction: Direction::Up,
            },
            Collider,
        ));
    }
}



fn snake_direction_change(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<&mut SnakeHead>) {
    if let Some(mut head) = query.iter_mut().next() {
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            if !matches!(head.direction, Direction::Right) {
                head.direction = Direction::Left;
            }
        } else if keyboard_input.pressed(KeyCode::ArrowRight) {
            if !matches!(head.direction, Direction::Left) {
                head.direction = Direction::Right;
            }
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            if !matches!(head.direction, Direction::Up) {
                head.direction = Direction::Down;
            }
        } else if keyboard_input.pressed(KeyCode::ArrowUp) {
            if !matches!(head.direction, Direction::Down) {
                head.direction = Direction::Up;
            }
        }
    }
}

fn snake_movement(mut windows: Query<&mut Window>, mut query_snake_head: Query<&mut SnakeHead>, mut query_transform: Query<(&mut Transform), With<SnakeHead>>, time: Res<Time>, mut timer: Local<SnakeMoveTimer>,) {
    timer.tick(time.delta());
    if timer.finished() {
        for window in windows.iter_mut() {
            for snake_head in query_snake_head.iter_mut() {
                for mut transform in query_transform.iter_mut() {
                    match snake_head.direction {
                        Direction::Left => {
                            transform.translation.x -= get_width(window.width());
                        }
                        Direction::Right => {
                            transform.translation.x += get_width(window.width());
                        }
                        Direction::Down => {
                            transform.translation.y -= get_height(window.height());
                        }
                        Direction::Up => {
                            transform.translation.y += get_height(window.height());
                        }
                    }
                }
            }
        }
    }
}

fn food_spawner(mut commands: Commands, mut windows: Query<&mut Window>, time: Res<Time>, mut timer: Local<FoodSpawnTimer>,) {
    timer.tick(time.delta());
    if timer.just_finished() {
        for window in windows.iter_mut() {
            let mut rng = thread_rng();
            let rand_position_x = rng.gen_range(0..ARENA_WIDTH);
            let translation_x = get_width(window.width()) / 2.0 + get_width(window.width()) * (rand_position_x - (ARENA_WIDTH / 2)) as f32;
            let rand_position_y = rng.gen_range(0..ARENA_HEIGHT);
            let translation_y = get_height(window.height()) / 2.0 + get_height(window.height()) * (rand_position_y - (ARENA_HEIGHT / 2)) as f32;

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(1.0, 0.0, 1.0, 1.0),
                        custom_size: Some(Vec2::new(get_width(window.width()), get_height(window.height()))),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(translation_x, translation_y, 0.0),
                        ..default()
                    },
                    ..default()
                },
                Food,
                Collider,
            ));
        }
    }
}

fn get_width(width: f32) -> f32 {
    width / ARENA_WIDTH as f32
}

fn get_height(height: f32) -> f32 {
    height / ARENA_HEIGHT as f32
}
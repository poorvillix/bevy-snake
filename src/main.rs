use bevy::prelude::*;
use rand::prelude::*;

fn main() {
    println!("Game Start!");

    println!("222");
    App::new()
        .add_systems(Startup, (setup).chain())
        .add_systems(Update, (snake_direction_change, food_spawner).chain())
        .add_systems(FixedUpdate, snake_movement)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Snake".to_string(),
                resolution: (400.0, 400.0).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .observe(body_follow_front)
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
    snake_body: Option<Entity>,
}

#[derive(Deref, DerefMut)]
struct SnakeMoveTimer(Timer);
impl Default for SnakeMoveTimer {
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
struct SnakeBody {
    snake_body: Option<Entity>,
}


#[derive(Component)]
struct Collider;



fn setup(mut commands: Commands, mut windows: Query<&mut Window>) {
    // camera
    commands.spawn(Camera2dBundle::default());

    // snake head
    let window = &windows.single_mut();
    let mut default_direction = Direction::Up;
    let first_body: Option<Entity> = match default_direction {
        Direction::Left => {
            Some(spawn_segment(&mut commands, window, get_width(window) / 2.0 + get_width(window), get_height(window) / 2.0))
        },
        Direction::Right => {
            Some(spawn_segment(&mut commands, window, get_width(window) / 2.0 - get_width(window), get_height(window) / 2.0))
        },
        Direction::Down => {
            Some(spawn_segment(&mut commands, window, get_width(window) / 2.0, get_height(window) / 2.0 + get_height(window)))
        },
        Direction::Up => {
            Some(spawn_segment(&mut commands, window, get_width(window) / 2.0, get_height(window) / 2.0 - get_height(window)))
        },
    };
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.7, 0.7, 0.7, 1.0),
                custom_size: Some(Vec2::new(get_width(window), get_height(window))),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(get_width(window) / 2.0, get_height(window) / 2.0, 0.0),
                scale: Vec3::new(0.8, 0.8, 1.0),
                ..default()
            },
            ..default()
        },
        SnakeHead {
            direction: default_direction,
            snake_body: first_body,
        },
        Collider,
    ));


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


// fn snake_movement(mut commands: Commands, mut windows: Query<&mut Window>, mut query_snake_head: Query<&mut SnakeHead>, mut query_head_transform: Query<&mut Transform, With<SnakeHead>>, bodies: &mut Query<&mut SnakeBody>, query_body_transform: &mut Query<&mut Transform, With<SnakeBody>>, mut timer: Local<SnakeMoveTimer>, time: Res<Time>,) {
fn snake_movement(mut commands: Commands, mut windows: Query<&mut Window>, mut query_snake_head: Query<&mut SnakeHead>, mut query_head_transform: Query<&mut Transform, With<SnakeHead>>, mut timer: Local<SnakeMoveTimer>, time: Res<Time>,) {
    timer.tick(time.delta());
    if timer.finished() {
        let window = &windows.single_mut();
        for mut transform in query_head_transform.iter_mut() {
            // body_movement(snake_head.snake_body, bodies, query_body_transform, transform.translation.x, transform.translation.y);
            let snake_head = query_snake_head.single_mut();
            if let Some(next_body) = snake_head.snake_body {
                println!("SnakeHead snake body: {:?}", snake_head.snake_body);
                commands.trigger(BackBody {
                    entity: next_body,
                    follow_position: Vec3::new(transform.translation.x, transform.translation.y, 0.0),
                });
            }
            match snake_head.direction {
                Direction::Left => {
                    transform.translation.x -= get_width(window);
                }
                Direction::Right => {
                    transform.translation.x += get_width(window);
                }
                Direction::Down => {
                    transform.translation.y -= get_height(window);
                }
                Direction::Up => {
                    transform.translation.y += get_height(window);
                }
            }
        }
    }
}

fn food_spawner(mut commands: Commands, mut windows: Query<&mut Window>, time: Res<Time>, mut timer: Local<FoodSpawnTimer>,) {
    timer.tick(time.delta());
    if timer.just_finished() {
        let window = &windows.single_mut();
        let mut rng = thread_rng();
        let rand_position_x = rng.gen_range(0..ARENA_WIDTH);
        let translation_x = get_width(window) / 2.0 + get_width(window) * (rand_position_x - (ARENA_WIDTH / 2)) as f32;
        let rand_position_y = rng.gen_range(0..ARENA_HEIGHT);
        let translation_y = get_height(window) / 2.0 + get_height(window) * (rand_position_y - (ARENA_HEIGHT / 2)) as f32;

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgba(1.0, 0.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(get_width(window), get_height(window))),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(translation_x, translation_y, 0.0),
                    scale: Vec3::new(0.8, 0.8, 1.0),
                    ..default()
                },
                ..default()
            },
            Food,
            Collider,
        ));

    }
}

fn spawn_segment(commands: &mut Commands, window: &Mut<Window>, x: f32, y: f32) -> Entity {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.7, 0.7, 0.7, 1.0),
                custom_size: Some(Vec2::new(get_width(window), get_height(window))),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(x, y, 0.0),
                scale: Vec3::new(0.65, 0.65, 1.0),
                ..default()
            },
            ..default()
        },
        SnakeBody {
            snake_body: None,
        },
        Collider,
    )).id()
}

#[derive(Event)]
struct BackBody {
    entity: Entity,
    follow_position: Vec3,
}

fn body_follow_front(trigger: Trigger<BackBody>, mut query_body: Query<&SnakeBody>, mut query_body_transform: Query<&mut Transform, With<SnakeBody>>, mut commands: Commands) {
    let event = trigger.event();
    if let Ok(snake_body) = query_body.get_mut(event.entity) {
        if let Ok(mut body_transform) = query_body_transform.get_mut(event.entity) {
            if let Some(next_body) = snake_body.snake_body {
                commands.trigger(BackBody {
                    entity: next_body,
                    follow_position: Vec3::new(body_transform.translation.x, body_transform.translation.y, 0.0),
                });
            }
            body_transform.translation.x = event.follow_position.x;
            body_transform.translation.y = event.follow_position.y;
            body_transform.translation.z = event.follow_position.z;
        }
    }
}

fn get_width(window: &Mut<Window>) -> f32 {
    window.width() / ARENA_WIDTH as f32
}

fn get_height(window: &Mut<Window>) -> f32 {
    window.height() / ARENA_HEIGHT as f32
}
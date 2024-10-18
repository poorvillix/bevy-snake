use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use rand::prelude::*;

fn main() {
    println!("Game Start!");
    App::new()
        .add_systems(Startup, (setup).chain())
        .add_systems(Update, (snake_direction_change, food_spawner).chain())
        .add_systems(FixedUpdate, (snake_movement, size_scaling, position_translation).chain())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Snake".to_string(),
                resolution: (400.0, 400.0).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .observe(body_follow_front)
        .observe(check_snake_eat_food)
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
    next_body: Option<Entity>,
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
    next_body: Option<Entity>,
}

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Size {
    scale: f32,
}

#[derive(Component)]
struct Collider;

#[derive(Event)]
struct NextBody {
    entity: Entity,
    follow_position: Position,
}

#[derive(Event)]
struct CheckSnakeEat {
    snake_position: Position,
}


fn setup(mut commands: Commands) {
    // camera
    commands.spawn(Camera2dBundle::default());
    let default_position_x = ARENA_WIDTH / 2;
    let default_position_y = ARENA_HEIGHT / 2;

    let default_direction = Direction::Up;
    let first_body: Option<Entity> = match default_direction {
        Direction::Left => {
            Some(spawn_segment(&mut commands, default_position_x + 1, default_position_y))
        },
        Direction::Right => {
            Some(spawn_segment(&mut commands, default_position_x - 1, default_position_y))
        },
        Direction::Down => {
            Some(spawn_segment(&mut commands, default_position_x, default_position_y + 1))
        },
        Direction::Up => {
            Some(spawn_segment(&mut commands, default_position_x, default_position_y - 1))
        },
    };
    // snake head
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.7, 0.7, 0.7, 1.0),
                ..default()
            },
            ..default()
        },
        SnakeHead {
            direction: default_direction,
            next_body: first_body,
        },
        Position {
            x: default_position_x,
            y: default_position_y,
        },
        Size {
            scale: 0.8
        },
        Collider,
    ));
}

fn size_scaling(mut windows: Query<&mut Window>, mut query: Query<(&Size, &mut Sprite)>) {
    let window = windows.single_mut();
    for (size, mut sprite) in &mut query.iter_mut() {
        sprite.custom_size = Some(Vec2::new(window.width() / ARENA_WIDTH as f32 * size.scale, window.height() / ARENA_HEIGHT as f32 * size.scale));
    }
}

fn position_translation(mut windows: Query<&mut Window>, mut query: Query<(&Position, &mut Transform)>) {
    fn convert(p: f32, bound_window: f32, bound_game: f32) -> f32 {
        p / bound_game * bound_window - (bound_window / 2.)
    }
    let window = windows.single_mut();
    for (pos, mut transform) in &mut query.iter_mut() {
        // transform.translation = Vec3::new(convert(pos.x as f32, window.width(), ARENA_WIDTH as f32), convert(pos.y as f32, window.height(), ARENA_HEIGHT as f32), 0.0);
        transform.translation.x = convert(pos.x as f32, window.width(), ARENA_WIDTH as f32);
        transform.translation.y = convert(pos.y as f32, window.height(), ARENA_HEIGHT as f32);
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

// fn snake_movement(mut commands: Commands, mut query_snake_head: Query<(&mut SnakeHead, &mut Position)>, mut query_food: Query<(Entity, &Position), With<Food>>, mut timer: Local<SnakeMoveTimer>, time: Res<Time>,) {
fn snake_movement(mut commands: Commands, mut query_snake_head: Query<(&mut SnakeHead, &mut Position)>, mut timer: Local<SnakeMoveTimer>, time: Res<Time>,) {
    timer.tick(time.delta());
    if timer.finished() {
        let (snake_head, mut position_snake_head) = query_snake_head.single_mut();

        let mut next_translation_x = position_snake_head.x;
        let mut next_translation_y = position_snake_head.y;
        match snake_head.direction {
            Direction::Left => {
                next_translation_x -= 1;
            }
            Direction::Right => {
                next_translation_x += 1;
            }
            Direction::Down => {
                next_translation_y -= 1;
            }
            Direction::Up => {
                next_translation_y += 1;
            }
        }

        commands.trigger(CheckSnakeEat {
            snake_position: Position {
                x:next_translation_x,
                y:next_translation_y},
        });

        if let Some(next_body) = snake_head.next_body {
            commands.trigger(NextBody {
                entity: next_body,
                follow_position: Position {
                    x:position_snake_head.x,
                    y:position_snake_head.y},
            });
        }

        position_snake_head.x = next_translation_x;
        position_snake_head.y = next_translation_y;
    }
}


// fn food_spawner(mut commands: Commands, time: Res<Time>, mut query_position: Query<(&mut Position)>, mut timer: Local<FoodSpawnTimer>,) {
//     timer.tick(time.delta());
//     if timer.just_finished() {
//         let mut rng = thread_rng();
//         let rand_position_x = rng.gen_range(0..ARENA_WIDTH);
//         let rand_position_y = rng.gen_range(0..ARENA_HEIGHT);
//
//         commands.spawn((
//             SpriteBundle {
//                 sprite: Sprite {
//                     color: Color::srgba(1.0, 0.0, 1.0, 1.0),
//                     ..default()
//                 },
//                 ..default()
//             },
//             Food,
//             Position {
//                 x: rand_position_x,
//                 y: rand_position_y,
//             },
//             Size {
//                 scale: 0.8
//             },
//             Collider,
//         ));
//     }
// }
fn food_spawner(mut commands: Commands, time: Res<Time>, mut query_position: Query<(&Position)>, mut timer: Local<FoodSpawnTimer>,) {
    timer.tick(time.delta());
    if timer.just_finished() {

        let mut numbers: Vec<i32> = (0..ARENA_WIDTH * ARENA_HEIGHT).collect();
        for (position) in &mut query_position.iter_mut() {
            numbers.retain(|&x| x != position.x + position.y * ARENA_HEIGHT);
        }
        println!("Numbers: {:?}", numbers);
        let mut rng = thread_rng();
        if numbers.len() > 0 {
            let rand = rng.gen_range(0..numbers.len());
            let rand_number = numbers.get(rand);
            if let Some(rand_number) = rand_number {
                let rand_position_x = rand_number % ARENA_WIDTH;
                let rand_position_y = rand_number / ARENA_HEIGHT;

                println!("rand_number = {} rand_number_x = {}, rand_number_y = {}", rand_number, rand_position_x, rand_position_y);
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(1.0, 0.0, 1.0, 1.0),
                            ..default()
                        },
                        ..default()
                    },
                    Food,
                    Position {
                        x: rand_position_x,
                        y: rand_position_y,
                    },
                    Size {
                        scale: 0.8
                    },
                    Collider,
                ));
            }
        }
    }
}

fn spawn_segment(commands: &mut Commands, position_x: i32, position_y: i32) -> Entity {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgba(0.7, 0.7, 0.7, 1.0),
                ..default()
            },
            ..default()
        },
        SnakeBody {
            next_body: None,
        },
        Position {
            x: position_x,
            y: position_y,
        },
        Size {
            scale: 0.65
        },
        Collider,
    )).id()
}

fn body_follow_front(trigger: Trigger<NextBody>, mut query_snake_body: Query<(&mut SnakeBody, &mut Position)>, mut commands: Commands) {
    let event = trigger.event();
    if let Ok((snake_body, mut position)) = query_snake_body.get_mut(event.entity) {
        if let Some(next_body) = snake_body.next_body {
            commands.trigger(NextBody {
                entity: next_body,
                follow_position: Position {
                    x: position.x,
                    y: position.y,
                },
            });
        }
        position.x = event.follow_position.x;
        position.y = event.follow_position.y;
    }
}

fn check_snake_eat_food(trigger: Trigger<CheckSnakeEat>, mut query_snake_head: Query<(&SnakeHead)>, mut query_snake_body: Query<(&mut SnakeBody)>, mut query_food: Query<(&mut Position, Entity), With<Food>>, mut commands: Commands) {
    let event = trigger.event();
    for (position, entity) in query_food.iter_mut() {
        if event.snake_position.x == position.x && event.snake_position.y == position.y {
            commands.entity(entity).despawn();
            let snake_head = query_snake_head.single_mut();
            let mut entity_next_body = snake_head.next_body;
            loop {
                if let Some(some_entity_next_body) = entity_next_body {
                    if let Ok(mut snake_body) = query_snake_body.get_mut(some_entity_next_body) {
                        if snake_body.next_body.is_some() {
                            entity_next_body = snake_body.next_body
                        } else {
                            snake_body.next_body = Some(spawn_segment(&mut commands, 0, 0));
                            break;
                        }
                    }
                }
            }
        }
    }
}


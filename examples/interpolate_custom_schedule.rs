//! This example showcases how interpolation can be implemented with this crate.

use bevy::{
    color::palettes::{
        css::WHITE,
        tailwind::{CYAN_400, RED_400},
    },
    ecs::schedule::ScheduleLabel,
    prelude::*,
};
use bevy_fixed_update_task::background_fixed_schedule::{
    BackgroundFixedUpdatePlugin, PostWriteBack, PreWriteBack, TaskResults, TaskToRenderTime,
    TaskWorker, Timestep,
};
use bevy_transform_interpolation::{
    prelude::*, RotationEasingState, ScaleEasingState, TransformEasingSet, TranslationEasingState,
};
use task_user::{AngularVelocity, LinearVelocity, TaskSleepTime, TaskWorkerTraitImpl, ToMove};

use std::time::Duration;

const MOVEMENT_SPEED: f32 = 250.0;
const ROTATION_SPEED: f32 = 2.0;

fn main() {
    let mut app = App::new();

    let easing_plugin = TransformEasingPlugin {
        schedule_fixed_first: PreWriteBack.intern(),
        schedule_fixed_last: PostWriteBack.intern(),
        schedule_fixed_loop: bevy::app::prelude::RunFixedMainLoop.intern(),
        after_fixed_main_loop: RunFixedMainLoopSystem::AfterFixedMainLoop.intern(),
        update_easing_values: false,
    };
    let interpolation_plugin = TransformInterpolationPlugin {
        schedule_fixed_first: PreWriteBack.intern(),
        schedule_fixed_last: PostWriteBack.intern(),
        interpolate_translation_all: false,
        interpolate_rotation_all: false,
        interpolate_scale_all: false,
    };

    // Add the `TransformInterpolationPlugin` to the app to enable transform interpolation.
    app.add_plugins((
        DefaultPlugins,
        BackgroundFixedUpdatePlugin::<task_user::TaskWorkerTraitImpl>::default(),
        easing_plugin,
        interpolation_plugin,
    ));

    // Setup the scene and UI, and update text in `Update`.
    app.add_systems(Startup, (setup, setup_text)).add_systems(
        bevy::app::prelude::RunFixedMainLoop,
        (
            change_timestep,
            change_sleep_time,
            update_timestep_text,
            update_sleep_time_text,
            update_diff_to_render_text,
        ),
    );

    app.add_systems(
        bevy::app::prelude::RunFixedMainLoop,
        (ease_translation_lerp, ease_rotation_slerp, ease_scale_lerp)
            .in_set(TransformEasingSet::Ease),
    );

    // Run the app.
    app.run();
}

/// Eases the translations of entities with linear interpolation.
fn ease_translation_lerp(
    mut query: Query<(&mut Transform, &TranslationEasingState)>,
    time: Query<(&TaskToRenderTime, &Timestep)>,
) {
    let Ok((time, timestep)) = time.get_single() else {
        return;
    };
    let overstep = (time.diff.max(0.0) / timestep.timestep.as_secs_f64()).min(1.0) as f32;
    query.iter_mut().for_each(|(mut transform, interpolation)| {
        if let (Some(start), Some(end)) = (interpolation.start, interpolation.end) {
            transform.translation = start.lerp(end, overstep);
        }
    });
}

/// Eases the rotations of entities with spherical linear interpolation.
fn ease_rotation_slerp(
    mut query: Query<(&mut Transform, &RotationEasingState)>,
    time: Query<(&TaskToRenderTime, &Timestep)>,
) {
    let Ok((time, timestep)) = time.get_single() else {
        return;
    };
    let overstep = (time.diff.max(0.0) / timestep.timestep.as_secs_f64()).min(1.0) as f32;

    query
        .par_iter_mut()
        .for_each(|(mut transform, interpolation)| {
            if let (Some(start), Some(end)) = (interpolation.start, interpolation.end) {
                // Note: `slerp` will always take the shortest path, but when the two rotations are more than
                // 180 degrees apart, this can cause visual artifacts as the rotation "flips" to the other side.
                transform.rotation = start.slerp(end, overstep);
            }
        });
}

/// Eases the scales of entities with linear interpolation.
fn ease_scale_lerp(
    mut query: Query<(&mut Transform, &ScaleEasingState)>,
    time: Query<(&TaskToRenderTime, &Timestep)>,
) {
    let Ok((time, timestep)) = time.get_single() else {
        return;
    };
    let overstep = (time.diff.max(0.0) / timestep.timestep.as_secs_f64()).min(1.0) as f32;

    query.iter_mut().for_each(|(mut transform, interpolation)| {
        if let (Some(start), Some(end)) = (interpolation.start, interpolation.end) {
            transform.scale = start.lerp(end, overstep);
        }
    });
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Spawn a camera.
    commands.spawn(Camera2d);

    let mesh = meshes.add(Rectangle::from_length(60.0));

    // Sets the "fake performance" to 1Hz.
    commands.insert_resource(TaskSleepTime(Duration::from_secs_f32(1.0)));
    commands.spawn((
        Timestep {
            // Set the fixed timestep to just 2 Hz for demonstration purposes.
            timestep: Duration::from_secs_f32(1.0 / 2.0),
        },
        TaskResults::<TaskWorkerTraitImpl>::default(),
        TaskWorker {
            worker: TaskWorkerTraitImpl {},
        },
    ));

    // This entity uses transform interpolation.
    commands.spawn((
        Name::new("Interpolation"),
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(Color::from(CYAN_400)).clone()),
        Transform::from_xyz(-500.0, 60.0, 0.0),
        TransformInterpolation,
        LinearVelocity(Vec2::new(MOVEMENT_SPEED, 0.0)),
        AngularVelocity(ROTATION_SPEED),
        ToMove,
    ));

    // This entity is simulated in `FixedUpdate` without any smoothing.
    commands.spawn((
        Name::new("No Interpolation"),
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(Color::from(RED_400)).clone()),
        Transform::from_xyz(-500.0, -60.0, 0.0),
        LinearVelocity(Vec2::new(MOVEMENT_SPEED, 0.0)),
        AngularVelocity(ROTATION_SPEED),
        ToMove,
    ));
}

/// Changes the timestep of the simulation when the up or down arrow keys are pressed.
fn change_timestep(mut time: Query<&mut Timestep>, keyboard_input: Res<ButtonInput<KeyCode>>) {
    let mut time = time.single_mut();
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        let new_timestep = (time.timestep.as_secs_f64() * 0.9).max(1.0 / 255.0);
        time.timestep = Duration::from_secs_f64(new_timestep);
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        let new_timestep = (time.timestep.as_secs_f64() * 1.1)
            .min(1.0)
            .max(1.0 / 255.0);
        time.timestep = Duration::from_secs_f64(new_timestep);
    }
}

/// Changes the timestep of the simulation when the up or down arrow keys are pressed.
fn change_sleep_time(
    mut sleep_time: ResMut<TaskSleepTime>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        let new_sleep_time = (sleep_time.0.as_secs_f64() * 0.9).max(1.0 / 255.0);
        sleep_time.0 = Duration::from_secs_f64(new_sleep_time);
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        let new_sleep_time = (sleep_time.0.as_secs_f64() * 1.1).min(1.0).max(1.0 / 255.0);
        sleep_time.0 = Duration::from_secs_f64(new_sleep_time);
    }
}

#[derive(Component)]
struct TimestepText;

#[derive(Component)]
struct SleepTimeText;

#[derive(Component)]
struct TaskToRenderTimeText;

fn setup_text(mut commands: Commands) {
    let font = TextFont {
        font_size: 20.0,
        ..default()
    };

    commands
        .spawn((
            Text::new("Fixed Hz: "),
            TextColor::from(WHITE),
            font.clone(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
        ))
        .with_child((TimestepText, TextSpan::default()));

    commands.spawn((
        Text::new("Change Timestep With Up/Down Arrow"),
        TextColor::from(WHITE),
        font.clone(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));
    commands.spawn((
        Text::new("Change simulation performance With Right/Left Arrow"),
        TextColor::from(WHITE),
        font.clone(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(35.0),
            right: Val::Px(10.0),
            ..default()
        },
    ));

    commands.spawn((
        Text::new("Interpolation"),
        TextColor::from(CYAN_400),
        font.clone(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    commands.spawn((
        Text::new("No Interpolation"),
        TextColor::from(RED_400),
        font.clone(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(75.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));

    commands
        .spawn((
            Text::new("Diff to render time: "),
            TextColor::from(WHITE),
            font.clone(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Px(10.0),
                ..default()
            },
        ))
        .with_child((TaskToRenderTimeText, TextSpan::default()));

    commands
        .spawn((
            Text::new("Fixed update logic computing time: "),
            TextColor::from(WHITE),
            font.clone(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(125.0),
                left: Val::Px(10.0),
                ..default()
            },
        ))
        .with_child((SleepTimeText, TextSpan::default()));
}

fn update_timestep_text(
    mut text: Single<&mut TextSpan, With<TimestepText>>,
    time: Query<&Timestep>,
) {
    let timestep = time.single().timestep.as_secs_f32();
    text.0 = format!("{:.2} ({timestep:.3}s)", timestep.recip());
}

fn update_sleep_time_text(
    mut text: Single<&mut TextSpan, With<SleepTimeText>>,
    sleep_time: Res<TaskSleepTime>,
) {
    let timestep = sleep_time.0.as_secs_f32();
    text.0 = format!("{:.2} Hz ({timestep:.3}s)", timestep.recip());
}

fn update_diff_to_render_text(
    mut text: Single<&mut TextSpan, With<TaskToRenderTimeText>>,
    task_to_render: Single<&TaskToRenderTime>,
) {
    text.0 = format!("{:.2}", task_to_render.diff);
}

pub mod task_user {
    use std::{slice::IterMut, time::Duration};

    use bevy::prelude::*;
    use bevy_fixed_update_task::background_fixed_schedule::TaskWorkerTrait;

    #[derive(Resource)]
    pub struct TaskSleepTime(pub Duration);

    #[derive(Debug, Clone, Default)]
    pub struct TaskWorkerTraitImpl;

    impl TaskWorkerTrait for TaskWorkerTraitImpl {
        type TaskExtractedData = TaskExtractedData;
        type TaskResultPure = Vec<(Entity, Transform, LinearVelocity, AngularVelocity)>;

        fn work(
            &self,
            _worker: Entity,
            mut input: TaskExtractedData,
            timestep: Duration,
            substep_count: u32,
        ) -> Vec<(Entity, Transform, LinearVelocity, AngularVelocity)> {
            let simulated_time = timestep * substep_count;
            // Simulate an expensive task
            std::thread::sleep(input.sleep_time);

            // Move entities in a fixed amount of time. The movement should appear smooth for interpolated entities.
            flip_movement_direction(
                input
                    .data
                    .iter_mut()
                    .map(|(_, transform, lin_vel, _)| (transform, lin_vel))
                    .collect::<Vec<_>>()
                    .iter_mut(),
            );
            movement(
                input
                    .data
                    .iter_mut()
                    .map(|(_, transform, lin_vel, _)| (transform, lin_vel))
                    .collect::<Vec<_>>()
                    .iter_mut(),
                simulated_time,
            );
            rotate(
                input
                    .data
                    .iter_mut()
                    .map(|(_, transform, _, ang_vel)| (transform, ang_vel))
                    .collect::<Vec<_>>()
                    .iter_mut(),
                simulated_time,
            );
            input.data
        }

        fn extract(&self, _worker_entity: Entity, world: &mut World) -> TaskExtractedData {
            // TODO: use a system rather than a world.
            let mut query = world.query_filtered::<
                            (Entity, &Transform, &LinearVelocity, &AngularVelocity),
                            With<ToMove>,
                        >();

            let transforms_to_move: Vec<(Entity, Transform, LinearVelocity, AngularVelocity)> =
                query
                    .iter(world)
                    .map(|(entity, transform, lin_vel, ang_vel)| {
                        (entity, transform.clone(), lin_vel.clone(), ang_vel.clone())
                    })
                    .collect();
            let sleep_time = world.get_resource::<TaskSleepTime>().unwrap().0;
            TaskExtractedData {
                data: transforms_to_move,
                sleep_time,
            }
        }

        fn write_back(
            &self,
            _worker_entity: Entity,
            result: bevy_fixed_update_task::background_fixed_schedule::TaskResult<Self>,
            mut world: &mut World,
        ) {
            let mut q_transforms =
                world.query_filtered::<(&mut Transform, &mut LinearVelocity), With<ToMove>>();
            for (entity, new_transform, new_lin_vel, _) in result.result_raw.result.iter() {
                if let Ok((mut transform, mut lin_vel)) = q_transforms.get_mut(&mut world, *entity)
                {
                    *transform = *new_transform;
                    *lin_vel = new_lin_vel.clone();
                }
            }
        }
    }

    #[derive(Debug, Component, Clone)]
    pub struct TaskExtractedData {
        pub data: Vec<(Entity, Transform, LinearVelocity, AngularVelocity)>,
        pub sleep_time: Duration,
    }

    /// The linear velocity of an entity indicating its movement speed and direction.
    #[derive(Component, Debug, Clone, Deref, DerefMut)]
    pub struct LinearVelocity(pub Vec2);

    /// The angular velocity of an entity indicating its rotation speed.
    #[derive(Component, Debug, Clone, Deref, DerefMut)]
    pub struct AngularVelocity(pub f32);

    #[derive(Component, Debug, Clone)]
    pub struct ToMove;

    /// Flips the movement directions of objects when they reach the left or right side of the screen.
    fn flip_movement_direction(query: IterMut<(&mut Transform, &mut LinearVelocity)>) {
        for (transform, lin_vel) in query {
            if transform.translation.x > 500.0 && lin_vel.0.x > 0.0 {
                lin_vel.0 = Vec2::new(-lin_vel.x.abs(), 0.0);
            } else if transform.translation.x < -500.0 && lin_vel.0.x < 0.0 {
                lin_vel.0 = Vec2::new(lin_vel.x.abs(), 0.0);
            }
        }
    }

    /// Moves entities based on their `LinearVelocity`.
    fn movement(query: IterMut<(&mut Transform, &mut LinearVelocity)>, delta: Duration) {
        let delta_secs = delta.as_secs_f32();
        for (transform, lin_vel) in query {
            transform.translation += lin_vel.extend(0.0) * delta_secs;
        }
    }

    /// Rotates entities based on their `AngularVelocity`.
    fn rotate(query: IterMut<(&mut Transform, &mut AngularVelocity)>, delta: Duration) {
        let delta_secs = delta.as_secs_f32();
        for (transform, ang_vel) in query {
            transform.rotate_local_z(ang_vel.0 * delta_secs);
        }
    }
}

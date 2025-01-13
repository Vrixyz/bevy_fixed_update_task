use std::time::Duration;

use bevy::{prelude::*, time::TimeUpdateStrategy};
use bevy_fixed_update_task::background_fixed_schedule::{
    BackgroundFixedUpdatePlugin, SubstepCount, TaskToRenderTime, TaskWorker, TaskWorkerTrait,
    Timestep,
};

#[test]
pub fn minimal_move_lose_time() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        BackgroundFixedUpdatePlugin::<TaskWorkerTraitImpl>::default(),
    ));

    app.add_systems(Startup, |mut commands: Commands| {
        commands.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0,
        )));
        commands.spawn((
            Timestep {
                timestep: Duration::from_secs_f32(1.0 / 60.0),
            },
            TaskWorker {
                worker: TaskWorkerTraitImpl {},
            },
            WriteBackCount::default(),
        ));
        commands.spawn((ToSimulate, Transform::default()));
    });

    for _ in 0..11 {
        app.update();
    }
    let world = app.world_mut();
    assert_eq!(
        10,
        world
            .query::<&WriteBackCount>()
            .iter(world)
            .next()
            .unwrap()
            .0
    );
    let task_to_render_time = world
        .query::<&TaskToRenderTime>()
        .iter(world)
        .next()
        .unwrap();
    assert!(
        2.33 < task_to_render_time.diff,
        "Task to render time should be greater than 2.33 (task behind render time), but it is: {}",
        task_to_render_time.diff
    );
}
#[test]
pub fn minimal_move_catch_back_time() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        BackgroundFixedUpdatePlugin::<TaskWorkerTraitImpl>::default(),
    ));

    app.add_systems(Startup, |mut commands: Commands| {
        commands.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 30.0,
        )));
        commands.spawn((
            Timestep {
                timestep: Duration::from_secs_f64(1.0 / 60.0),
            },
            SubstepCount(60),
            TaskWorker {
                worker: TaskWorkerTraitImpl {},
            },
            WriteBackCount::default(),
        ));
        commands.spawn((ToSimulate, Transform::default()));
    });

    for _ in 0..121 {
        app.update();
    }
    let world = app.world_mut();
    assert_eq!(
        4,
        world
            .query::<&WriteBackCount>()
            .iter(world)
            .next()
            .unwrap()
            .0
    );
    let task_to_render_time = world
        .query::<&TaskToRenderTime>()
        .iter(world)
        .next()
        .unwrap();
    assert!(
        task_to_render_time.diff < 0.01,
        "Task to render time should be lower than 0.01, but it is: {}",
        task_to_render_time.diff
    );
}

#[derive(Component, Default)]
pub struct WriteBackCount(pub usize);

#[derive(Component)]
pub struct ToSimulate;

#[derive(Debug, Clone, Default)]
pub struct TaskWorkerTraitImpl;

impl TaskWorkerTrait for TaskWorkerTraitImpl {
    type TaskExtractedData = TaskExtractedData;
    type TaskResultPure = TaskExtractedData;

    fn work(
        &self,
        _worker: Entity,
        mut input: TaskExtractedData,
        timestep: Duration,
        substep_count: u32,
    ) -> Self::TaskResultPure {
        for i in input.positions.iter_mut() {
            i.1.translation += Vec3::new(timestep.as_secs_f32() * substep_count as f32, 0.0, 0.0);
        }
        input
    }

    fn extract(&self, _worker_entity: Entity, world: &mut World) -> TaskExtractedData {
        let positions = world
            .query_filtered::<(Entity, &Transform), With<ToSimulate>>()
            .iter(world)
            .map(|(e, t)| (e, t.clone()))
            .collect::<Vec<_>>();
        TaskExtractedData { positions }
    }

    fn write_back(
        &self,
        worker_entity: Entity,
        result: bevy_fixed_update_task::background_fixed_schedule::TaskResult<Self>,
        world: &mut World,
    ) {
        for (e, t) in result.result_raw.result.positions.iter() {
            world.get_mut::<Transform>(*e).unwrap().translation = t.translation;
        }
        world
            .query::<&mut WriteBackCount>()
            .get_mut(world, worker_entity)
            .unwrap()
            .0 += 1;
    }
}

#[derive(Debug, Component, Clone)]
pub struct TaskExtractedData {
    pub positions: Vec<(Entity, Transform)>,
}

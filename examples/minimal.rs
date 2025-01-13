//! This is a minimal example to show how synchronized fixed update works.

use bevy::prelude::*;
use bevy_fixed_update_task::{
    BackgroundFixedUpdatePlugin, TaskResults, TaskWorker, TaskWorkerTrait, Timestep,
};

use std::time::Duration;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        BackgroundFixedUpdatePlugin::<TaskWorkerTraitImpl>::default(),
    ));
    app.init_resource::<SimulationTime>();
    app.add_systems(Update, print_simulation_time);
    app.add_systems(Startup, setup_worker);

    // Run the app.
    app.run();
}

fn setup_worker(mut commands: Commands) {
    commands.spawn((
        Timestep {
            timestep: Duration::from_secs_f32(1.0 / 60.0),
        },
        TaskResults::<TaskWorkerTraitImpl>::default(),
        TaskWorker {
            worker: TaskWorkerTraitImpl {},
        },
    ));
}

fn print_simulation_time(simulation_time: Res<SimulationTime>, time: Res<Time>) {
    println!(
        "Simulation time: {:?} ; time: {:?}",
        simulation_time.time,
        time.elapsed()
    );
}

#[derive(Resource, Debug, Clone, Default)]
pub struct SimulationTime {
    pub time: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct TaskWorkerTraitImpl;

impl TaskWorkerTrait for TaskWorkerTraitImpl {
    type TaskExtractedData = TaskExtractedData;
    type TaskResultPure = Duration;

    fn work(
        &self,
        _worker: Entity,
        mut input: TaskExtractedData,
        timestep: Duration,
        _substep_count: u32,
    ) -> Self::TaskResultPure {
        std::thread::sleep(Duration::from_secs_f32(0.1));
        input.time += timestep;
        input.time
    }

    fn extract(&self, _worker_entity: Entity, world: &mut World) -> TaskExtractedData {
        let time = world.resource::<SimulationTime>();
        TaskExtractedData { time: time.time }
    }

    fn write_back(
        &self,
        _worker_entity: Entity,
        result: bevy_fixed_update_task::TaskResult<Self>,
        world: &mut World,
    ) {
        world.get_resource_mut::<SimulationTime>().unwrap().time = result.result_raw.result;
    }
}

#[derive(Debug, Component, Clone)]
pub struct TaskExtractedData {
    pub time: Duration,
}

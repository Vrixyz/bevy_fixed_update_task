# Physics timestep loop

Let's assume we want to make a game running at 60 frames per second, and use physics simulations.

In the ideal world, running everything sequentially allows us to finish below the "1/60" seconds allotted time frame.

When running complex physics simulations, the simulation step can be expensive, and not interruptible (you can't run half a simulation step).

After benchmarking, you realize your simulation step is way too expensive to be able to sustain 60 frames per seconds.

You have a few choices:

- Tweak all the knobs of your physics engine

Alter the physics engine configuration to see if some can yield to improvement.
It often means tuning down precision in favor of speed, a valid choice for a lot of projects!

- Reduce your frame per seconds to 30 :fear:

This will double your time budget! Great! But now, the game isn't just as smooth as before...

- Run the physics step in parallel to the whole frame loop.

You realize there's nothing you can do to make your physics simulation step fit in your time budget,
even more so if we want an option for 120 frames per second.

The solution is to run the physics timestep in parallel to the whole frame, and read its output later:

"But if the physics doesn't happen every frames, my moving objects will appear to stutter?"

Yes. That's where interpolation becomes interesting:

Let's try 30 physics simulations per seconds: when receiving physics output, we're doing a few things:

1. Wait for the rendering time to catch up on the simulated time.
2. start a new physics step, simulating "1/30" seconds.
3. start for each moving abject, a transition from previous position to the newly computed one.
   - This transition will end when next physics step is handled, because we're using "fixed" time steps, we know how long it takes!

If we don't wait on rendering time, we would be running the simulation faster than real time, and interpolation would end abruptly,
as we can't know for sure when the task will end.

To be exact, we *could* start a new physics step as soon as we received the previous one, but this would need a careful thought about not pre-simulating too much.

"But When a player hits a button, I want the physics to react!"

It's still possible!
But you're right to be suspicious: this technique involves at least 1 frame delay,
which might not be fitting very fast paced environment.

That being said, we can still register inputs whenever they occur, and display instant visual feedbacks,
even if the physics step isn't done yet. So this technique can still look very responsive.

This crate `bevy_fixed_update_task` is an implementation of a fixed update not throttled by rendering.

It has been originally made for `bevy_rapier`, but it might be useful to any other continuous task.

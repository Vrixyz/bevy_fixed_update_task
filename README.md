# Bevy fixed update task

Bevy's fixed update is throttled by rendering, but it doesn´t have to be!

This crate allows you to run a fixed update in a background task,
so you can improve your time budget.

[Read more about it.](docs/physics_timestep_loop.md)

## Practical advantage

Regain control over your frame per seconds: know when your expensive tasks are making you fall behind, and be able to adapt accordingly.

### Practical example

Using [rapier](https://rapier.rs/), simulation step can be expensive, leading to lags, like this example:

<video src="https://github.com/user-attachments/assets/f9c0ab20-3726-43cc-ac1e-b34860483857" height="350"></video>

Using this crate, you can simulate the expensive task in background, allowing to keep a steady visual frame per second.

<video src="https://github.com/user-attachments/assets/9215f0d6-a6d5-4a35-ab1a-45d300a607f4" height="350"></video>

Both these recordings are simulating 80 000 bodies targeting a fixed update of 30 frames per second.

Pair this technique with interpolation or other visual feedbacks to obtain a very responsive feeling.

## How it works

:warning: This crate makes most sense when using bevy's `multi_threaded` feature. Otherwise, this just adds unnecessary overhead.

This crate adds a custom scheduling comparable to bevy's fixed update, but eagerly extracts ECS data into a background task, to synchronize it only when `Time<Virtual>` catches back to the "simulated time".

The implementation doesn't use `Time<Fixed>` but a component approach `TimeStep`, `SubstepCount`, `TaskToRender`.

By relying on scheduling for extracting data and writing back, it's easier to order systems correctly when using other ecosystem crates such as `bevy_transform_interpolation`.

![image](https://github.com/user-attachments/assets/a1e2d3ac-eebe-4b3f-89ca-879024c0c740)

<details><summary>mermaid</summary>
<p>

Unfortunately mermaid has a few bugs and github doesn't rely on latest mermaid, you can paste that is https://mermaid.live for a better formatting:

```mermaid_raw
gantt
    title Background fixed update
    dateFormat  X
    axisFormat %L
    section Frames
        Frame 1                             :f1, 0, 0.16s
        Frame 2                             :f2,after f1, 0.16s
        Frame 3                             :f3,after f2, 0.16s
    section Bevy ecs
        Start a fixed update                :s1, 0, 0.001s
        Extract data                        :after s1, 0.01s
        Should we finish the fixed update?  :c1,after f1, 0.001s
        Should we finish the fixed update?  :c2,after f2, 0.001s
        Write back data                     :w1,after c2, 0.01s
        Start a new fixed update            :s2,after w1, 0.001s
        Extract data                        :e2,after s2, 0.01s
    section Background thread
        Background task                     :after e1, 0.25s
        Background task                     :after e2, 0.25s
```

</p>
</details> 

## Resources

- http://gameprogrammingpatterns.com/game-loop.html

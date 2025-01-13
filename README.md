# Bevy fixed update task

Bevy's fixed update is throttled by rendering, but it doesn´t have to be!

This crate allows you to run a fixed update in a background task,
so you can improve your time budget.

[Read more about it.](docs/physics_timestep_loop.md)

## How it works

:warning: this crate makes most sense when using bevy's `multi_threaded` feature. Otherwise, this just adds unnecessary overhead.

It's quite similar to how bevy's fixed update works, but eagerly extracts ECS data into a background task, to synchronize it only when we exceed `Time<Virtual>` + its accumulated time.

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

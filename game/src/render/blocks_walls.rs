use std::collections::BinaryHeap;

use crate::{
    SCREEN_HEIGHT,
    game::{
        Game,
        map::{LEVEL_HEIGHT, ShapeType},
    },
    render::{
        RendererData,
        camera_view::{
            ColumnTasks, RenderTask, RenderTaskOrderer, RenderTaskType, VerticalDisctance,
        },
        raycast::{BlockSlice, MapSlice, RayHit},
    },
};

pub fn task_column(
    game: &Game,
    renderer_data: &RendererData,
    map_slice: &MapSlice,
    angle_relative_to_player: f64,
) -> ColumnTasks {
    let mut tasks: BinaryHeap<RenderTaskOrderer> = BinaryHeap::new();

    if let Some(wall_hit) = &map_slice.wall_hit {
        tasks.push(task_side(
            &wall_hit,
            angle_relative_to_player,
            renderer_data,
            game,
        )); // default return value: empty column
    }

    for slice in &map_slice.block_slices {
        tasks.append(&mut task_block_slice(
            slice,
            angle_relative_to_player,
            renderer_data,
            game,
        ));
    }

    for exit_hit in &map_slice.hits_blocks_currently_inside {
        if let Some(task_ord) =
            task_partial_surface(exit_hit, angle_relative_to_player, renderer_data, game)
        {
            tasks.push(task_ord);
        }
    }

    let mut wall_distance = f64::MAX;
    if let Some(wh) = &map_slice.wall_hit {
        wall_distance = wh.distance;
    }

    return ColumnTasks {
        tasks,
        wall_distance,
    };
}

pub fn task_side(
    side_hit: &RayHit,
    angle_relative_to_player: f64,
    renderer_data: &RendererData,
    game: &Game,
) -> RenderTaskOrderer {
    let (side_bottom_onscreen, side_top_onscreen) =
        calculate_side_bottom_top(&side_hit, angle_relative_to_player, renderer_data, game);

    let color = side_hit.side.shape.color;
    // match &side_hit.side.shape.shape_type {
    //     ShapeType::Wall => renderer_data.wall_default_color,
    //     ShapeType::Block => renderer_data.block_default_color,
    // };

    let brightness = (side_hit.side.angle_in_world.cos() * 0.5
        / (side_hit.distance * renderer_data.distance_darkness_coefficient)
        + 0.5)
        .clamp(0.2, 1.0);

    let task = RenderTask {
        color,
        brightness,
        onscreen_bottom: side_bottom_onscreen,
        onscreen_top: side_top_onscreen,
    };

    return RenderTaskOrderer::new(task, side_hit.distance, RenderTaskType::Side);
}

pub fn task_surface(
    slice: &BlockSlice,
    angle_relative_to_player: f64,
    renderer_data: &RendererData,
    game: &Game,
) -> Option<RenderTaskOrderer> {
    let (exit_bottom_onscreen, exit_top_onscreen) = calculate_side_bottom_top(
        &slice.exit_hit,
        angle_relative_to_player,
        renderer_data,
        game,
    );
    let (entry_bottom_onscreen, entry_top_onscreen) = calculate_side_bottom_top(
        &slice.entry_hit,
        angle_relative_to_player,
        renderer_data,
        game,
    );

    let onscreen_dimensions: Option<(isize, isize)> = match &slice.entry_hit.side.shape.shape_type {
        ShapeType::Block => {
            if &slice.entry_hit.side.shape.bottom > &game.player.view_height {
                Some((exit_bottom_onscreen, entry_bottom_onscreen))
            } else if (&slice.entry_hit.side.shape.bottom + &slice.entry_hit.side.shape.height)
                < game.player.view_height
            {
                Some((entry_top_onscreen, exit_top_onscreen))
            } else {
                None
            }
        }
        ShapeType::Wall => None, // null value, shoud never happen
    };

    let vertical_distance: Option<VerticalDisctance> = match &slice.entry_hit.side.shape.shape_type
    {
        ShapeType::Block => {
            // case ceiling
            if &slice.entry_hit.side.shape.bottom > &game.player.view_height {
                Some(&slice.entry_hit.side.shape.bottom - &game.player.view_height)
            //case floor
            } else if (&slice.entry_hit.side.shape.bottom + &slice.entry_hit.side.shape.height)
                < game.player.view_height
            {
                Some(
                    game.player.view_height
                        - (&slice.entry_hit.side.shape.bottom + &slice.entry_hit.side.shape.height),
                )
            } else {
                None
            }
        }
        ShapeType::Wall => None, // null value, should never happen
    };

    // varies between 0.5 and 1.0 depending on height in level; temporary
    let brightness = 0.5 + (&slice.entry_hit.side.shape.height / LEVEL_HEIGHT) * 0.5;

    if let Some((onscreen_bottom, onscreen_top)) = onscreen_dimensions
        && let Some(vertical_distance_value) = vertical_distance
    {
        let task: RenderTask = RenderTask {
            color: slice.entry_hit.side.shape.surface_color,
            brightness,
            onscreen_bottom: onscreen_bottom,
            onscreen_top: onscreen_top,
        };

        //println!("{}|{}", task.onscreen_bottom, task.onscreen_top);

        let task_type: RenderTaskType = match &slice.entry_hit.side.shape.shape_type {
            ShapeType::Block => {
                if &slice.entry_hit.side.shape.bottom > &game.player.view_height {
                    RenderTaskType::Ceiling(vertical_distance_value)
                } else if (&slice.entry_hit.side.shape.bottom + &slice.entry_hit.side.shape.height)
                    < game.player.view_height
                {
                    RenderTaskType::Floor(vertical_distance_value)
                } else {
                    return None;
                }
            }
            ShapeType::Wall => {
                return None;
            }
        };

        return Some(RenderTaskOrderer::new(
            task,
            slice.exit_hit.distance,
            task_type,
        ));
    } else {
        return None;
    }
}

// TODO optimize? kinda laggy for some reason rn;
// TODO i know its this function lagging because if i comment out the invocation in task_column al lot less lag spikes happen
pub fn task_partial_surface(
    exit_hit: &RayHit,
    angle_relative_to_player: f64,
    renderer_data: &RendererData,
    game: &Game,
) -> Option<RenderTaskOrderer> {
    // if we are inside the block (no just horizontalll, but also vertically)
    if exit_hit.side.shape.bottom < game.player.view_height
        && exit_hit.side.shape.bottom + exit_hit.side.shape.height > game.player.view_height
    {
        return None;
    }

    let (exit_bottom_onscreen, exit_top_onscreen) =
        calculate_side_bottom_top(&exit_hit, angle_relative_to_player, renderer_data, game);

    let brightness = 0.5 + (&exit_hit.side.shape.height / LEVEL_HEIGHT) * 0.5;

    // if we are above the block
    if exit_hit.side.shape.bottom + exit_hit.side.shape.height < game.player.view_height {
        let vert_dist =
            game.player.view_height - exit_hit.side.shape.bottom + exit_hit.side.shape.height;
        let task: RenderTask = RenderTask {
            color: exit_hit.side.shape.surface_color,
            brightness: brightness,
            onscreen_bottom: 0,
            onscreen_top: exit_top_onscreen,
        };
        return Some(RenderTaskOrderer {
            task: task,
            task_type: RenderTaskType::Floor(vert_dist),
            distance: exit_hit.distance,
        });
    } else {
        // otherwise we are below the block
        let vert_dist = exit_hit.side.shape.bottom - game.player.view_height;
        let task: RenderTask = RenderTask {
            color: exit_hit.side.shape.surface_color,
            brightness: brightness,
            onscreen_bottom: exit_bottom_onscreen,
            onscreen_top: SCREEN_HEIGHT as isize,
        };
        return Some(RenderTaskOrderer {
            task: task,
            task_type: RenderTaskType::Floor(vert_dist),
            distance: exit_hit.distance,
        });
    }
}

pub fn task_block_slice(
    slice: &BlockSlice,
    angle_relative_to_player: f64,
    renderer_data: &RendererData,
    game: &Game,
) -> BinaryHeap<RenderTaskOrderer> {
    let mut tasks = BinaryHeap::new();

    tasks.push(task_side(
        &slice.entry_hit,
        angle_relative_to_player,
        renderer_data,
        game,
    ));

    if let Some(task_surface_value) =
        task_surface(slice, angle_relative_to_player, renderer_data, game)
    {
        tasks.push(task_surface_value);
    }

    tasks
}

pub fn calculate_side_bottom_top(
    rh: &RayHit,
    angle_relative_to_player: f64,
    renderer_data: &RendererData,
    game: &Game,
) -> (isize, isize) {
    let normalized_distance_to_side = rh.distance * angle_relative_to_player.cos(); // cos for anti-fisheye effect

    let side_height_onscreen = ((rh.side.shape.height / normalized_distance_to_side)
        * renderer_data.render_scale_coefficient) as isize; // must be addable to bottom_onscreen

    let mut side_bottom_onscreen: isize = ((renderer_data.screen_height_as_f64 / 2.0) // middle of screen
        + ((rh.side.shape.bottom / normalized_distance_to_side)
        - (game.player.view_height / normalized_distance_to_side)) // adjust for view hieght
        * renderer_data.render_scale_coefficient) // scale correctly
        as isize;

    let side_top_onscreen =
        (side_bottom_onscreen + side_height_onscreen).min(SCREEN_HEIGHT as isize);
    side_bottom_onscreen = side_bottom_onscreen.max(0);

    (side_bottom_onscreen, side_top_onscreen)
}

use bevy::prelude::*;
use std::fs;

#[derive(Component)]
struct Voxel;

#[derive(Component)]
struct VoxelId(u32);

#[derive(Component)]
struct Line(VoxelId, f32);

#[derive(Component)]
struct Vel(f32, f32);

#[derive(Component)]
struct Acc(f32, f32);

#[derive(Component)]
struct Del(f32);

#[derive(Component)]
struct IsFixed(bool);


fn setup(mut commands : Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}


fn spawn_voxels(mut commands : Commands) {
    let file = fs::read_to_string(r"E:\Rust Projects\phys2d\src\Voxel.json").expect("Unable to read file");
    let json: serde_json::Value = serde_json::from_str(file.as_str()).expect("JSON was not well-formatted");
    let mut id = 0;
    for i in json.as_array().unwrap() {
        let mut vox = commands.spawn_bundle(SpriteBundle {
            sprite : Sprite {
                color : Color::rgb(
                    i["Color"]["r"].as_f64().unwrap() as f32 / 255., 
                    i["Color"]["g"].as_f64().unwrap() as f32 / 255., 
                    i["Color"]["b"].as_f64().unwrap() as f32 / 255.),
                ..Default::default()
            },
            transform : Transform {
                scale : Vec3::new(10., 10., 10.),
                translation : Vec3::new(
                    i["pos"][0].as_f64().unwrap() as f32, 
                    i["pos"][1].as_f64().unwrap() as f32, 
                    0.),
                ..Default::default()
            },
            ..Default::default()
        });
        vox
        .insert(Voxel)
        .insert(Vel(
            i["vel"][0].as_f64().unwrap() as f32, 
            i["vel"][0].as_f64().unwrap() as f32
        ))
        .insert(Acc(
            i["acc"][0].as_f64().unwrap() as f32, 
            i["acc"][0].as_f64().unwrap() as f32 - 1.
        ))
        .insert(VoxelId(id))
        .insert(IsFixed(i["is fixed"].as_bool().unwrap()));
        for j in i["Line"].as_array().unwrap(){
            vox.insert(Line(
                VoxelId(j["Id"].as_i64().unwrap() as u32),
                j["Length"].as_f64().unwrap() as f32,
            ));
        }
        id += 1;
    }
}

fn update (
    win_desc: Res<WindowDescriptor>,
    delta : Res<Del>,
    mut voxel_set : Query<(&mut Transform, &mut Vel, &Acc, &IsFixed)>
) {
    for _i in 0..((1. / delta.0) as i32) {

        for (mut vox, mut vel, acc, is_fixed) in voxel_set.iter_mut() {
            if !is_fixed.0 {
                let mut nx = vel.0 * delta.0 + vox.translation.x;
                let mut ny = vel.1 * delta.0 + vox.translation.y;
                if nx.abs() + vox.scale.x / 2.> win_desc.width / 2. {
                    vel.0 = -vel.0 * 0.999; // TODO : Make this coff a res
                    nx = vox.translation.x;
                }
                if ny.abs() + vox.scale.y / 2.> win_desc.height / 2. {
                    vel.1 = -vel.1 * 0.999;
                    ny = vox.translation.y;
                }

                vox.translation.x = nx;
                vox.translation.y = ny;
                
                // Update Forces

                vel.0 = vel.0 * (1. - 0.001 * delta.0); // Drag
                vel.1 = vel.1 * (1. - 0.001 * delta.0); // TODO : Make this coff a res

                vel.0 += acc.0;
                vel.1 += acc.1;
            }
        }
    }
}


// TODO : merge update_constraints with update for reproducability
fn update_constraints (mut voxel_set : Query<(&mut Transform, &mut Vel, &Line, &IsFixed)>){
    let mut pos: Vec<(f32, f32, Vel, &Line, &IsFixed)> = Vec::new();

    for (vox, vel, line, is_fixed) in voxel_set.iter(){
        pos.push((vox.translation.x, vox.translation.y, Vel(vel.0, vel.1), line, is_fixed));
    }

    for i in 0..pos.len(){
        let line_id = pos[i].3.0.0 as usize;
        let line_distance = pos[i].3.1 as f32;
        let distance = ((pos[i].0 - pos[line_id].0).powf(2.) + (pos[i].1 - pos[line_id].1).powf(2.)).sqrt();
        let diff = (distance - line_distance) / distance / 2.;
        
        // Original position
        let posx1 = pos[i].0 - pos[i].2.0;
        let posy1 = pos[i].1 - pos[i].2.1;
        
        let posx2 = pos[line_id].0 - pos[line_id].2.0;
        let posy2 = pos[line_id].1 - pos[line_id].2.1;


        // Deltas
        let dx = pos[i].0 - pos[line_id].0;
        let dy = pos[i].1 - pos[line_id].1;

        
        // Update position
        if !pos[i].4.0 { 
            pos[i].0 -= dx * diff; 
            pos[i].1 -= dy * diff;
        }

        if !pos[line_id].4.0 { 
            pos[line_id].0 += dx * diff;
            pos[line_id].1 += dy * diff;
        }


        // Update velocity
        pos[i].2.0 = pos[i].0 - posx1;
        pos[i].2.1 = pos[i].1 - posy1;

        pos[line_id].2.0 = pos[line_id].0 - posx2;
        pos[line_id].2.1 = pos[line_id].1 - posy2;
    }

    let mut i = 0;

    for (mut vox, mut vel, _line, _is_fixed) in voxel_set.iter_mut() {
        vox.translation.x = pos[i].0;
        vox.translation.y = pos[i].1;

        vel.0 = pos[i].2.0;
        vel.1 = pos[i].2.1;
        i += 1;
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "2D Physics".to_string(),
            width: 500.0,
            height: 500.0,
            ..Default::default()
        })
        .insert_resource(Del(1.))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(spawn_voxels)
        .add_system(update)
        .add_system(update_constraints)
        .run();
}

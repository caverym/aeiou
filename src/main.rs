#![windows_subsystem = "windows"]
mod prelude;

use std::time::Duration;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum AeiouState {
    Loading,
    Play,
}

#[derive(AssetCollection)]
struct AeiouAssets {
    #[asset(path = "anim.png")]
    atlas: Handle<Image>,
    #[asset(path = "mus.mp3")]
    music: Handle<bevy_kira_audio::AudioSource>,
    #[asset(path = "media.png")]
    keyatlas: Handle<Image>,
    #[asset(path = "playerball.png")]
    ball: Handle<Image>,
    #[asset(path = "line.png")]
    line: Handle<Image>,
}

fn main() {
    println!("aeiou");
    App::new()
        .insert_resource(WindowDescriptor {
            width: 498.0,
            height: 498.0,
            title: "aeiou".to_string(),
            resizable: false,
            cursor_visible: false,
            ..default()
        })
        .add_loading_state(
            LoadingState::new(AeiouState::Loading)
                .continue_to_state(AeiouState::Play)
                .with_collection::<AeiouAssets>(),
        )
        .add_state(AeiouState::Loading)
        .add_plugins_with(DefaultPlugins, |group| {
            group.add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
        })
        .add_plugin(AudioPlugin)
        // .add_startup_system(setup)
        .add_system_set(
            SystemSet::on_enter(AeiouState::Play)
                .with_system(play)
                .with_system(setup),
        )
        .add_system_set(
            SystemSet::on_update(AeiouState::Play)
                .with_system(playpause)
                .with_system(bganim)
                .with_system(update),
        )
        .add_audio_channel::<Channel>()
        .run();
}

struct Channel;

#[derive(Component)]
struct AeiouTimer(Timer);

#[derive(Component)]
struct Media;

#[derive(Component)]
struct Ball;

struct Ai(Handle<AudioInstance>);

fn playpause(
    channel: Res<AudioChannel<Channel>>,
    keys: Res<Input<KeyCode>>,
    mut instance: ResMut<Ai>,
    aias: Res<AeiouAssets>,
    mut query: Query<&mut TextureAtlasSprite, With<Media>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        match channel.state(&instance.0) {
            PlaybackState::Paused { .. }
            | PlaybackState::Pausing { .. }
            | PlaybackState::Queued => {
                set_media(&mut query, AeiouMedia::Play);
                channel.resume();
            }
            PlaybackState::Playing { .. } => {
                set_media(&mut query, AeiouMedia::Paused);
                channel.pause();
            }
            PlaybackState::Stopped | PlaybackState::Stopping { .. } => {
                set_media(&mut query, AeiouMedia::Play);
                instance.0 = channel.play(aias.music.clone()).handle();
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum AeiouMedia {
    Paused = 0,
    Play = 1,
}

fn set_media(query: &mut Query<&mut TextureAtlasSprite, With<Media>>, state: AeiouMedia) {
    for mut image in query {
        image.index = (state as u8) as usize;
    }
}

fn bganim(time: Res<Time>, mut query: Query<(&mut AeiouTimer, &mut TextureAtlasSprite)>) {
    for (mut tmr, mut atl) in &mut query {
        if tmr.0.tick(time.delta()).just_finished() {
            let mut index = atl.index + 1;

            if index == 63 {
                index -= 63;
            }

            atl.index = index;
        }
    }
}

fn play(
    mut cmds: Commands,
    channel: Res<AudioChannel<Channel>>,
    aias: Res<AeiouAssets>,
) {
    cmds.insert_resource(Ai(channel.play(aias.music.clone()).handle()));
}

//  1824
fn update(mut query: Query<&mut Transform, With<Ball>>, channel: Res<AudioChannel<Channel>>, instance: Res<Ai>) {
    if let Some(pc) = channel.state(&instance.0).position().map(|f| (f as f32) / 1824.) {
        let pos = (482. * (pc / 100.)) - 233.;
        for mut trans in &mut query {
            trans.translation.x = pos as f32;
        }
    }
}

fn setup(mut cmds: Commands, aias: Res<AeiouAssets>, mut textures: ResMut<Assets<TextureAtlas>>) {
    cmds.spawn_bundle(SpriteSheetBundle {
        texture_atlas: textures.add(TextureAtlas::from_grid(
            aias.atlas.clone(),
            Vec2::splat(498.0),
            7,
            9,
        )),
        ..default()
    })
    .insert(AeiouTimer(Timer::new(Duration::from_millis(100), true)));

    cmds.spawn_bundle(SpriteSheetBundle {
        texture_atlas: textures.add(TextureAtlas::from_grid(
            aias.keyatlas.clone(),
            Vec2::splat(64.0),
            2,
            1,
        )),
        transform: Transform {
            translation: Vec3 {
                x: 0.0,
                y: -166.0,
                z: 1.0,
            },
            ..default()
        },
        sprite: TextureAtlasSprite {
            index: 1,
            ..default()
        },
        ..default()
    })
    .insert(Media);

    cmds.spawn_bundle(
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-164.0, -233.0, 1.0),
                ..default()
            },
            texture: aias.ball.clone(),
            ..default()
        }
    ).insert(Ball);

    cmds.spawn_bundle(
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0., -233.0, 1.0),
                ..default()
            },
            texture: aias.line.clone(),
            ..default()
        }
    );

    cmds.spawn_bundle(Camera2dBundle::default());
}

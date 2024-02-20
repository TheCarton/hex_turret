use std::iter::Cycle;

use bevy::prelude::*;

use crate::enemies::{
    CurrentFireflyAnimationState, DamagedTime, Firefly, FireflyAnimationState,
    PrevFireflyAnimationState,
};

pub(crate) struct HexTurretAnimationPlugin;

impl Plugin for HexTurretAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate_sprite,
                update_firefly_animation_state,
                update_firefly_animation,
            ),
        );
    }
}

#[derive(Component, Debug)]
pub(crate) struct AnimationIndices {
    pub(crate) cycle: Cycle<std::ops::Range<usize>>,
}

impl Default for AnimationIndices {
    fn default() -> Self {
        AnimationIndices {
            cycle: (0usize..1usize).into_iter().cycle(),
        }
    }
}

impl AnimationIndices {
    pub(crate) fn new(first: usize, last: usize) -> AnimationIndices {
        AnimationIndices {
            cycle: (first..last).cycle(),
        }
    }

    pub(crate) fn next_index(&mut self) -> usize {
        self.cycle.next().expect("cycle never empty")
    }

    pub(crate) fn firefly_indices() -> AnimationIndices {
        AnimationIndices {
            cycle: (0..3).cycle(),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct AnimationTimer {
    pub(crate) timer: Timer,
}

impl Default for AnimationTimer {
    fn default() -> Self {
        AnimationTimer {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut indices, mut timer, mut sprite) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = indices.next_index();
        }
    }
}

fn update_firefly_animation(
    mut q_fireflies: Query<(
        &CurrentFireflyAnimationState,
        &mut PrevFireflyAnimationState,
        &mut AnimationIndices,
        Changed<CurrentFireflyAnimationState>,
    )>,
) {
    for (curr_anim, mut prev_anim, mut indices, _) in q_fireflies.iter_mut() {
        if curr_anim.state != prev_anim.state {
            *indices = match curr_anim.state {
                FireflyAnimationState::Normal => AnimationIndices::new(0, 3),
                FireflyAnimationState::Damaged => AnimationIndices::new(16, 19),
            };
            prev_anim.state = curr_anim.state;
        }
    }
}

fn update_firefly_animation_state(
    mut q_fireflies: Query<(
        &mut CurrentFireflyAnimationState,
        &mut PrevFireflyAnimationState,
        &mut DamagedTime,
        With<Firefly>,
    )>,
    time: Res<Time>,
) {
    //TODO: Fix logic for transition from normal animation cycle to hit animation cycle. We're going to
    // incorrect animation indices right now.
    for (mut animation_state, mut prev_animation_state, mut hit_timer, _) in q_fireflies.iter_mut()
    {
        if let Some(timer) = &mut hit_timer.time {
            timer.tick(time.delta());
            if timer.finished() {
                *animation_state = CurrentFireflyAnimationState {
                    state: FireflyAnimationState::Normal,
                };
                *prev_animation_state = PrevFireflyAnimationState {
                    state: FireflyAnimationState::Damaged,
                };
                hit_timer.time = None;
            } else {
                *animation_state = CurrentFireflyAnimationState {
                    state: FireflyAnimationState::Damaged,
                };
            }
        }
    }
}

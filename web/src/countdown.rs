//! The AI round's countdown, big in the middle of the screen: 3, 2, 1,
//! then FIGHT (user, 2026-10-10). Empty for the copy and the box.

use crate::sim_step::SimRes;
use bevy::prelude::*;
use drivesim_physics::Sim;
use drivesim_physics::opponent::COUNTDOWN_S;

/// How long "FIGHT" stays up after the AI starts, s.
pub const FIGHT_SHOWN_S: f32 = 1.0;

#[derive(Component)]
pub struct CountdownText;

pub fn spawn_countdown(commands: &mut Commands) {
    commands.spawn((
        CountdownText,
        Text::new(""),
        TextFont {
            font_size: bevy::text::FontSize::Px(96.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.45, 0.08)),
        TextLayout::justify(bevy::text::Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.0),
            width: Val::Percent(100.0),
            ..default()
        },
    ));
}

/// What the banner says for `s`: whole seconds left, "FIGHT", or nothing.
pub fn banner(s: &Sim) -> String {
    let ai = s.opponent.as_ref().is_some_and(|o| o.ai.is_some());
    match s.countdown_left() {
        Some(left) => format!("{}", left.ceil() as u32),
        None if ai && s.time_s < COUNTDOWN_S + FIGHT_SHOWN_S => "FIGHT".into(),
        None => String::new(),
    }
}

pub fn update_countdown(sim: Option<Res<SimRes>>, mut q: Query<&mut Text, With<CountdownText>>) {
    let Ok(mut text) = q.single_mut() else {
        return;
    };
    let want = sim.map(|s| banner(&s.sim)).unwrap_or_default();
    if **text != want {
        **text = want;
    }
}

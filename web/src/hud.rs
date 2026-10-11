//! The heads-up display: what the driver can't see from the box but needs
//! while practising (radio, arming, speed, weapon speed, tilt).

use crate::radio::Radio;
use crate::sim_step::SimRes;
use bevy::prelude::*;

#[derive(Component)]
pub struct HudText;

pub fn spawn_hud(commands: &mut Commands) {
    commands.spawn((
        HudText,
        Text::new("loading robot…"),
        TextFont {
            font_size: bevy::text::FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

pub fn update_hud(
    sim: Option<Res<SimRes>>,
    radio: Res<Radio>,
    mut q: Query<&mut Text, With<HudText>>,
) {
    let (Some(sim), Ok(mut text)) = (sim, q.single_mut()) else {
        return;
    };
    let s = &sim.sim;
    let c = s.chassis();
    let up = *c.rotation() * rapier3d::prelude::Vector::Z;
    let link = match (&radio.name, radio.frame.is_some()) {
        _ if s.pilot.is_some() => "AI driving (AI vs AI)",
        (Some(_), true) => "radio connected",
        _ => "NO RADIO (move a stick so the browser sees it)",
    };
    let armed = if s.duties.left.is_some() || s.duties.weapon.is_some() {
        "ARMED"
    } else {
        "disarmed"
    };
    let weapon = s
        .model
        .weapons
        .first()
        .map(|w| {
            format!(
                "{:.0} rpm",
                s.rotor_speed(w).abs() * 60.0 / std::f32::consts::TAU
            )
        })
        .unwrap_or_else(|| "none".into());
    let v = c.linvel();
    **text = format!(
        "{}\n{link} · {armed}\nspeed {:.1} m/s · weapon {weapon} · tilt {:.0}°\nphysics {:.1}× real time",
        s.model.name,
        (v.x * v.x + v.y * v.y).sqrt(),
        up.z.clamp(-1.0, 1.0).acos().to_degrees(),
        sim.headroom,
    );
}

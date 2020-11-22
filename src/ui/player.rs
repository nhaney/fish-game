// use bevy::prelude::*;
//
// use crate::shared::game::Score;
//
// pub(super) struct PlayerCountdownText;
//
// pub(super) fn add_score_text(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands
//         .spawn(TextComponents {
//             style: Style {
//                 align_self: AlignSelf::FlexEnd,
//                 ..Default::default()
//             },
//             text: Text {
//                 value: "Score:".to_string(),
//                 font: asset_server.load("fonts/FiraSans-Bold.ttf"),
//                 style: TextStyle {
//                     font_size: 60.0,
//                     color: Color::GREEN,
//                     ..Default::default()
//                 },
//             },
//             ..Default::default()
//         })
//         .with(ScoreText);
// }
//
// pub(super) fn update_score_text(score: Res<Score>, mut query: Query<(&mut Text, &ScoreText)>) {
//     for (mut text, _) in query.iter_mut() {
//         text.value = format!("Score: {:?}", score.count);
//     }
// }

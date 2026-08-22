use bevy::camera::visibility::RenderLayers;
use bevy::camera::{Camera, Camera2d, ClearColorConfig};
use bevy::color::Color;
use bevy::ecs::{
    bundle::Bundle,
    hierarchy::ChildOf,
    system::{Commands, Res, ResMut}
};
use bevy::text::{FontSize, TextColor, TextFont};
use bevy::ui::{percent, widget::Text, AlignContent, AlignItems, AlignSelf, BackgroundColor, BorderRadius, Display, FlexDirection, FocusPolicy, GridTrack, IsDefaultUiCamera, JustifyContent, JustifySelf, Node, UiRect, Val};
use bevy::utils::default;

use crate::game::{LayoutData, CLUE_INDICES, SPARE_INDICES, PLANE_INDICES};
use crate::render::{GameSeedLabel, Theme};

pub fn setup_layout(
    mut commands: Commands,
    theme: Res<Theme>,
    mut data: ResMut<LayoutData>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            ..default()
        },
        IsDefaultUiCamera,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(1),
    ));


    let mut grid_template_columns = Vec::new();
    for _ in PLANE_INDICES {
        grid_template_columns.push(GridTrack::fr(1.2));
        for _ in CLUE_INDICES {
            grid_template_columns.push(GridTrack::fr(1.0));
        }
    }

    let mut grid_template_rows = vec![
        GridTrack::fr(1.2),
    ];
    for _ in PLANE_INDICES {
        grid_template_rows.push(GridTrack::fr(1.0));
    }

    let parent_id = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: percent(100),
            height: percent(100),
            align_content: AlignContent::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )).id();

    let board_id = commands.spawn((
        Node {
            width: percent(100),
            height: percent(100),
            display: Display::Grid,
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            grid_template_columns,
            grid_template_rows,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.2, 0.1)),
        ChildOf(parent_id),
    )).id();

    fn make_clue() -> impl Bundle {
        Node {
            display: Display::Flex,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        }
    }

    fn make_card() -> impl Bundle {
        (
            Node {
                display: Display::Flex,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                width: Val::Px(80.0),
                border: UiRect::top(Val::Px(2.0)).with_left(Val::Px(2.0)),
                margin: UiRect::right(Val::Px(2.0)).with_bottom(Val::Px(40.0)),
                padding: UiRect::all(Val::Px(8.0)).with_right(Val::Px(0.0)),
                border_radius: BorderRadius::top_left(Val::Px(10.0)),
                ..default()
            },
            FocusPolicy::Block,
        )
    }

    fn make_spare() -> impl Bundle {
        (
            Node {
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Px(80.0),
                border: UiRect::all(Val::Px(2.0)).with_bottom(Val::Px(0.0)),
                padding: UiRect::all(Val::Px(8.0)),
                margin: UiRect::all(Val::Px(4.0)),
                border_radius: BorderRadius::top(Val::Px(10.0)),
                ..default()
            },
            FocusPolicy::Block,
        )
    }

    commands.spawn((
        Node::default(),
        ChildOf(board_id),
        GameSeedLabel,
    ));

    for i in CLUE_INDICES {
        data.top_ids[i] = commands.spawn((
            make_clue(),
            ChildOf(board_id),
        )).id();
    }

    commands.spawn((
        Node::default(), ChildOf(board_id),
    ));

    for i in CLUE_INDICES {
        data.bottom_ids[i] = commands.spawn((
            make_clue(),
            ChildOf(board_id),
        )).id();
    }

    for i in CLUE_INDICES {
        data.left_ids[i] = commands.spawn((
            make_clue(),
            ChildOf(board_id),
        )).id();
        for j in CLUE_INDICES {
            data.tile_ids[i][j][0] = commands.spawn((
                make_card(),
                ChildOf(board_id),
            )).id();
        }
        data.right_ids[i] = commands.spawn((
            make_clue(),
            ChildOf(board_id),
        )).id();
        for j in CLUE_INDICES {
            data.tile_ids[i][j][1] = commands.spawn((
                make_card(),
                ChildOf(board_id),
            )).id();
        }
    }

    let footer_id = commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            ..default()
        },
        ChildOf(parent_id),
    )).id();

    // Instructions at bottom
    let instructions_id = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ChildOf(footer_id),
    )).id();

    commands.spawn((
        Node {
            ..default()
        },
        ChildOf(instructions_id),
        Text::new("R - redeal; Click to guess specific card"),
        TextColor(Color::srgb(0.8, 0.6, 0.6)),
        TextFont::from(theme.font.clone()).with_font_size(FontSize::Px(24.0)),
    ));
    commands.spawn((
        Node {
            ..default()
        },
        ChildOf(instructions_id),
        Text::new("(C,D,H,S) - guess suit; (2-10,J,Q,K,A) - guess value; Space - clear guess"),
        TextColor(Color::srgb(0.8, 0.6, 0.6)),
        TextFont::from(theme.font.clone()).with_font_size(FontSize::Px(24.0)),
    ));

    // Spares in the bottom-right corner
    commands.spawn((
        Node {
            ..default()
        },
        Text::new("Spares:"),
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        TextFont::from(theme.font.clone()).with_font_size(FontSize::Px(24.0)),
        ChildOf(footer_id),
    ));

    for i in SPARE_INDICES {
        data.spare_ids[i] = commands.spawn((
            make_spare(),
            ChildOf(footer_id),
        )).id();
    }
}

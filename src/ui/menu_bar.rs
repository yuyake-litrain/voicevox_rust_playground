use iced::widget::button::Status;
use iced::widget::{button, row};
use iced::{Alignment, Background, Border, Color, Length, Theme, color};
use iced_aw::style::menu_bar;
use iced_aw::widget::menu::{Item, Menu, MenuBar};


use crate::{IcedVVGUIState, ui::Message};

pub fn menu_bar<'a>(state: &IcedVVGUIState) -> iced::widget::Row<'a, Message, Theme> {
    let style = |_: &Theme, status: button::Status| match status {
        Status::Disabled => button::Style {
            background: None,
            text_color: color!(0x444444),
            ..Default::default()
        },
        Status::Active => button::Style {
            background: None,
            text_color: Color::WHITE,
            ..Default::default()
        },
        Status::Hovered => button::Style {
            background: Some(Background::Color(color!(0x222222))),
            text_color: Color::WHITE,
            ..Default::default()
        },
        Status::Pressed => button::Style {
            background: Some(Background::Color(color!(0x444444))),
            text_color: Color::WHITE,
            ..Default::default()
        },
    };
    row!({
        let root_file = Item::with_menu(
            button("File")
                .style(style)
                .on_press(Message::MenuBarBtnPressed),
            Menu::new(vec![
                Item::new(button("Open Project").style(style)),
                Item::new(button("Close").style(style)),
                Item::with_menu(
                    button("Recent Projects").style(style),
                    Menu::new(vec![
                        Item::new(button("hoge_prj1").style(style)),
                        Item::new(button("hoge_prj2").style(style)),
                        Item::new(button("hoge_prj3").style(style)),
                    ]),
                ),
                Item::new(button("Export as WAV").style(style).on_press_maybe(
                    match state.able_to_tts {
                        true => Some(Message::TTSBtnPressed),
                        false => None,
                    },
                )),
                Item::new(button("Save").style(style)),
                Item::new(button("Save as...").style(style)),
                Item::new(button("Exit").style(style)),
            ]),
        );

        let root_edit = Item::with_menu(
            button("Edit")
                .style(style)
                .on_press(Message::MenuBarBtnPressed),
            Menu::new(vec![
                Item::new(
                    button("Say")
                        .style(style)
                        .on_press_maybe(match state.able_to_tts {
                            true => Some(Message::TTSBtnPressed),
                            false => None,
                        }),
                ),
                Item::new(button("Cut").style(style)),
                Item::new(button("Copy").style(style)),
                Item::new(button("Paste").style(style)),
                Item::new(button("Select All").style(style)),
                Item::new(button("Remove").style(style)),
                Item::new(button("Settings...").style(style)),
            ]),
        );

        let root_view = Item::with_menu(
            button("View")
                .style(style)
                .on_press(Message::MenuBarBtnPressed),
            Menu::new(vec![Item::new(button("Set default").style(style))]),
        );
        let root_engine = Item::with_menu(
            button("Engine")
                .style(style)
                .on_press(Message::MenuBarBtnPressed),
            Menu::new(vec![
                Item::new(button("Engines...").style(style)),
                Item::new(button("Models...").style(style)),
            ]),
        );
        let root_plugins = Item::with_menu(
            button("Plugins").style(style),
            Menu::new(vec![Item::new(button("Not Implemented").style(style))]),
        );
        MenuBar::new(vec![
            root_file,
            root_edit,
            root_view,
            root_engine,
            root_plugins,
        ])
        .style(|_, _| menu_bar::Style {
            menu_border: Border {
                color: color!(0x444444),
                width: 1.0,
                ..Default::default()
            },
            bar_background: Background::Color(Color::TRANSPARENT),
            menu_background: Background::Color(color!(0x0F0F0F)),
            ..Default::default()
        })
    },)
    .width(Length::Fill)
    .align_y(Alignment::Start)
}

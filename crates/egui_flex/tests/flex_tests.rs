use eframe::emath::Vec2;
use egui::{
    Align, Button, Checkbox, Context, DragValue, Frame, Label, Layout, ScrollArea, TextEdit, Ui,
};
use egui_flex::{item, Flex, FlexAlign, FlexAlignContent, FlexItem, FlexJustify, Size};
use egui_kittest::wgpu::TestRenderer;
use egui_kittest::Harness;
use hello_egui_utils::run;
use rstest::rstest;
use std::cell::Cell;

fn snapshot_name() -> String {
    let thread = std::thread::current();
    let thread_name = thread.name().unwrap();
    thread_name.replace("::", "_")
}

fn should_be_stable(harness: &mut Harness) {
    let first = TestRenderer::new().render(harness);

    for _ in 0..3 {
        harness.run();
        let second = TestRenderer::new().render(harness);
        #[allow(clippy::manual_assert)]
        if first != second {
            panic!("Is not stable");
        };
    }
}

#[rstest]
fn test_justify(
    #[values(false, true)] grow: bool,
    #[values(
        FlexAlign::Start,
        FlexAlign::Center,
        FlexAlign::End,
        FlexAlign::Stretch
    )]
    align: FlexAlign,
) {
    let justify_values = [
        FlexJustify::Start,
        FlexJustify::Center,
        FlexJustify::End,
        FlexJustify::SpaceBetween,
        FlexJustify::SpaceAround,
        FlexJustify::SpaceEvenly,
    ];

    let mut app = |ui: &mut Ui| {
        ui.label(format!("align: {align:?}, grow: {grow}"));

        for justify in &justify_values {
            ui.group(|ui| {
                ui.label(format!("{justify:?}"));

                let mut flex = egui_flex::Flex::horizontal()
                    .height(40.0)
                    .align_items(align);

                if grow {
                    flex = flex.grow_items(1.0);
                }

                flex.justify(*justify).w_full().show(ui, |flex| {
                    for _ in 0..3 {
                        flex.add(item(), Button::new("Label"));
                    }
                });
            });
        }
    };

    let mut harness = Harness::new_ui(app);

    harness.wgpu_snapshot(&snapshot_name());
}

#[test]
fn test_insert_remove() {
    let show = Cell::new(false);

    let mut harness = Harness::new_ui(|ui| {
        Flex::horizontal()
            .w_full()
            .grow_items(1.0)
            .show(ui, |flex| {
                flex.add(item(), Label::new("Label"));
                if show.get() {
                    flex.add(item(), Label::new("New\nLabel\nMultiline"));
                }
                flex.add(item(), Label::new("Label 2"));
            });
    });

    let mut results = vec![];

    results.push(harness.try_wgpu_snapshot("test_insert_remove_0"));

    show.set(true);
    harness.run();
    results.push(harness.try_wgpu_snapshot("test_insert_remove_1"));

    show.set(false);
    harness.run();
    results.push(harness.try_wgpu_snapshot("test_insert_remove_2"));

    for result in results {
        result.unwrap();
    }
}

#[rstest]
fn test_size(
    #[values(
        None,
        Some(Size::Points(100.0)),
        Some(Size::Percent(0.5)),
        Some(Size::Percent(1.0))
    )]
    width: Option<Size>,
    #[values(
        None,
        Some(Size::Points(100.0)),
        Some(Size::Percent(0.5)),
        Some(Size::Percent(1.0))
    )]
    height: Option<Size>,
) {
    let mut harness = Harness::new_ui(|ui| {
        ui.group(|ui| {
            let mut flex = Flex::horizontal();

            if let Some(width) = width {
                flex = flex.width(width);
            }
            if let Some(height) = height {
                flex = flex.height(height);
            }

            flex.justify(FlexJustify::Center)
                .align_items(FlexAlign::Center)
                .show(ui, |flex| {
                    flex.add(item(), Button::new("Button"));
                });
        });
    });

    harness.wgpu_snapshot(&snapshot_name());
}

#[test]
#[ignore]
fn basis_stabilize() {
    let mut harness = Harness::new_ui(|ui| {
        Flex::horizontal()
            .justify(FlexJustify::Center)
            .wrap(false)
            .show(ui, |flex| {
                flex.add(item(), Button::new("Button 1"));
                flex.add(
                    item(),
                    Label::new("Button 12398983274892379847239847293873489"),
                );
            });

        Flex::horizontal().grow_items(1.0).show(ui, |flex| {
            flex.add_flex(
                item().basis(100.0),
                Flex::horizontal().justify(FlexJustify::Center),
                |flex| {
                    flex.add(item(), Button::new("Button 1"));
                    flex.add(
                        item(),
                        Label::new("Button 12398983274892379847239847293873489"),
                    );
                },
            );

            flex.add(item().basis(200.0), Button::new("Button 2"));
        });
    });

    should_be_stable(&mut harness);

    harness.wgpu_snapshot("basis_stabilize");
}

#[test]
fn nested() {
    let mut harness = Harness::new_ui(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(10.0);
        let frame = Frame::group(ui.style());
        Flex::horizontal()
            .align_content(FlexAlignContent::Start)
            .grow_items(1.0)
            .show(ui, |flex| {
                flex.add_flex_frame(
                    FlexItem::new(),
                    Flex::vertical()
                        .align_content(FlexAlignContent::Stretch)
                        .h_full()
                        .grow_items(1.0),
                    Frame::group(flex.ui().style()),
                    |flex| {
                        flex.add(FlexItem::new(), Button::new("btn"));
                        // flex.add(
                        //     FlexItem::new(),
                        //     Slider::new(&mut flt, 0.0..=1000.0).show_value(false),
                        // );
                        flex.add(
                            FlexItem::new().grow(0.0),
                            TextEdit::singleline(&mut String::new()).desired_width(100.0),
                        );
                        flex.add(FlexItem::new(), DragValue::new(&mut 0.0));
                        flex.add(FlexItem::new(), Checkbox::new(&mut false, "Checkbox"));
                    },
                );

                flex.add(FlexItem::new().grow(1.0), Button::new("Single Button"));

                flex.add_flex_frame(
                    FlexItem::new().grow(1.0),
                    Flex::vertical()
                        .align_content(FlexAlignContent::Stretch)
                        .grow_items(1.0)
                        .h_full(),
                    frame,
                    |flex| {
                        flex.add(FlexItem::new().grow(1.0), Button::new("btn"));
                        flex.add(FlexItem::new(), Button::new("Very long button"));
                    },
                );

                flex.add_flex_frame(
                    FlexItem::new().grow(1.0),
                    Flex::vertical()
                        .align_content(FlexAlignContent::Stretch)
                        .grow_items(1.0),
                    frame,
                    |flex| {
                        flex.add_flex_frame(
                            FlexItem::new().grow(1.0),
                            Flex::horizontal()
                                .align_content(FlexAlignContent::Stretch)
                                .grow_items(1.0),
                            frame,
                            |flex| {
                                flex.add(FlexItem::new().grow(1.0), Button::new("btn"));
                                flex.add(FlexItem::new(), Button::new("Very long button"));

                                flex.add_flex_frame(
                                    FlexItem::new().grow(1.0),
                                    Flex::vertical()
                                        .align_content(FlexAlignContent::Stretch)
                                        .grow_items(1.0),
                                    Frame::group(flex.ui().style()),
                                    |flex| {
                                        flex.add(FlexItem::new().grow(1.0), Button::new("btn"));
                                        flex.add(FlexItem::new(), Button::new("Very long button"));
                                    },
                                );
                            },
                        );

                        flex.add(FlexItem::new().grow(1.0), Button::new("btn"));
                        flex.add(FlexItem::new(), Button::new("Very long button"));
                        flex.add(FlexItem::new(), Button::new("btn"));
                    },
                );
            });

        Flex::vertical().show(ui, |flex| {
            flex.add_flex_frame(
                FlexItem::new(),
                Flex::horizontal()
                    .align_content(FlexAlignContent::Start)
                    .grow_items(1.0),
                Frame::group(flex.ui().style()),
                |flex| {
                    flex.add(FlexItem::new().grow(1.0), Button::new("btn"));
                    flex.add(FlexItem::new(), Button::new("Very long button"));
                },
            );
        });
    });

    should_be_stable(&mut harness);

    harness.wgpu_snapshot("nested");
}

// Tests the interaction with vertical_centered_justified
#[test]
fn egui_justify_interaction() {
    let mut harness = Harness::new_ui(|ui| {
        ui.group(|ui| {
            ui.label("vertical_centered_justified");
            ui.vertical_centered_justified(|ui| {
                ui.button("Justified normal button");

                Flex::vertical().show(ui, |flex| {
                    flex.add(item(), Button::new("Justified flex button (vertical)"));
                });

                Flex::horizontal().show(ui, |flex| {
                    flex.add(
                        item().grow(1.0),
                        Button::new("Justified flex button (horizontal)"),
                    );
                });
            });
        });

        ui.group(|ui| {
            ui.label("vertical normal");

            ui.button("Non-justified normal button");
            Flex::horizontal().show(ui, |flex| {
                flex.add(item().grow(1.0), Button::new("Non-justified flex button"));
            });
        });
    });

    should_be_stable(&mut harness);

    harness.wgpu_snapshot("egui_justify_interaction");
}

// This somewhat matches the chat ui in HelloPaint, but the test seems to be broken currently
#[test]
pub fn chat() {
    let mut harness = Harness::builder().with_size([300.0, 200.0]).build_ui(|ui| {
        Flex::vertical()
            .h_full()
            .w_full()
            .align_content(FlexAlignContent::Stretch)
            .show(ui, |flex| {
                flex.add_ui(FlexItem::new().grow(1.0).basis(0.0), |ui| {
                    ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.label("Messages");
                        ui.label("Messages");
                        ui.label("Messages");
                        ui.label("Messages");
                        ui.label("Messages");
                        ui.label("Messages");
                        ui.label("Messages");
                        ui.label("Messages");
                        ui.label("Messages");

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            ui.label("My message");
                        });
                    });
                });

                let frame = Frame::none()
                    //.fill(flex.ui().visuals().faint_bg_color)
                    .inner_margin(8.0);
                flex.add_flex_frame(
                    FlexItem::new(),
                    Flex::horizontal().w_full(),
                    frame,
                    |flex| {
                        flex.add_flex(item().grow(1.0), Flex::horizontal(), |flex| {
                            flex.add(
                                item().grow(1.0).basis(0.0),
                                TextEdit::singleline(&mut String::new()),
                            );
                            flex.add(item(), Button::new("Send"));
                        });
                    },
                );
            });
    });

    should_be_stable(&mut harness);

    harness.wgpu_snapshot("chat");
}

#[test]
fn truncate() {
    let texts = ["Hello", "Helloooooooooooooooooooooooooooooooo"];

    let text_index = Cell::new(0);
    let mut harness = Harness::builder().with_size([300.0, 600.0]).build_ui(|ui| {
        let text = texts[text_index.get()];

        ui.group(|ui| {
            ui.set_max_width(100.0);
            ui.set_max_height(80.0);

            ui.separator();

            Flex::vertical().w_full().h_full().show(ui, |flex| {
                let frame = Frame::group(flex.ui().style());
                flex.add_flex_frame(item(), Flex::horizontal(), frame, |flex| {
                    flex.add(item().shrink(), Button::new(text).wrap());
                    flex.add(item(), Button::new("World!").wrap());
                });
            });
        });

        Flex::horizontal().w_full().show(ui, |flex| {
            flex.add_flex(item().shrink(), Flex::horizontal(), |flex| {
                flex.add(item().shrink(), Button::new(text).wrap());
            });
            flex.add(item(), Button::new("World!"));
        });

        Flex::horizontal().w_full().show(ui, |flex| {});

        ui.add(Button::new("Helloooooooooooooooooooooo").truncate());
        Flex::horizontal().w_full().show(ui, |flex| {
            flex.add_ui(item().shrink(), |ui| {
                ui.add(Button::new(text).wrap());
            });

            flex.add(item(), Button::new("World!"));
        });
        Flex::horizontal().w_full().show(ui, |flex| {
            flex.add(item().shrink(), Button::new(text).wrap());
            flex.add(item(), Button::new("World!"));
        });
    });

    let mut results = vec![];

    harness.run();
    should_be_stable(&mut harness);
    results.push(harness.try_wgpu_snapshot("truncate_0_short"));

    text_index.set(1);
    harness.run();
    harness.run();
    should_be_stable(&mut harness);
    results.push(harness.try_wgpu_snapshot("truncate_1_long"));

    text_index.set(0);
    harness.run();
    harness.run();
    should_be_stable(&mut harness);
    results.push(harness.try_wgpu_snapshot("truncate_2_short"));

    for result in results {
        result.unwrap();
    }
}

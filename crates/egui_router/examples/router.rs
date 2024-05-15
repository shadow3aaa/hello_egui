use eframe::NativeOptions;
use egui::{CentralPanel, Color32, Context, Frame, Ui, Window};
use egui_inbox::type_inbox::TypeInbox;
use egui_router::{EguiRouter, Request, Route, TransitionConfig};

struct AppState {
    message: String,
    inbox: TypeInbox,
}

enum RouterMessage {
    Navigate(String),
    Back,
}

fn main() -> eframe::Result<()> {
    let init = |ctx: &Context| {
        let mut router = EguiRouter::new(AppState {
            message: "Hello, World!".to_string(),
            inbox: TypeInbox::new(ctx.clone()),
        })
        .with_backward_transition(
            TransitionConfig::slide().with_easing(egui_animation::easing::quad_in_out),
        )
        .with_forward_transition(
            TransitionConfig::slide().with_easing(egui_animation::easing::quad_in_out),
        )
        .with_default_duration(0.3);

        router.route("/", home);
        router.route("/edit", edit_message);
        router.route("/post/{id}", post);

        router.navigate_transition("/", TransitionConfig::none());

        router
    };

    let mut router: Option<EguiRouter<AppState>> = None;
    let mut window_router: Option<EguiRouter<AppState>> = None;

    eframe::run_simple_native(
        "Router Example",
        NativeOptions::default(),
        move |ctx, frame| {
            let mut router = router.get_or_insert_with(|| init(ctx));
            let mut window_router = window_router.get_or_insert_with(|| init(ctx));

            for router in [&mut router, &mut window_router].iter_mut() {
                router
                    .state
                    .inbox
                    .read()
                    .for_each(|msg: RouterMessage| match msg {
                        RouterMessage::Navigate(route) => {
                            router.navigate(route);
                        }
                        RouterMessage::Back => {
                            router.back();
                        }
                    });
            }

            CentralPanel::default().show(ctx, |ui| {
                router.ui(ui);
            });

            Window::new("Router Window")
                .frame(Frame::window(&ctx.style()).inner_margin(0.0))
                .show(ctx, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height());
                    window_router.ui(ui);
                });
        },
    )
}

fn home(request: &mut Request<AppState>) -> impl Route<AppState> {
    |ui: &mut Ui, state: &mut AppState| {
        background(ui, ui.style().visuals.faint_bg_color, |ui| {
            ui.heading("Home!");

            ui.label(format!("Message: {}", state.message));

            if ui.link("Edit Message").clicked() {
                state
                    .inbox
                    .send(RouterMessage::Navigate("/edit".to_string()));
            }

            ui.label("Navigate to post:");

            if ui.link("Post 1").clicked() {
                state
                    .inbox
                    .send(RouterMessage::Navigate("/post/1".to_string()));
            }

            if ui.link("Post 2").clicked() {
                state
                    .inbox
                    .send(RouterMessage::Navigate("/post/2".to_string()));
            }

            if ui.link("Invalid Post").clicked() {
                state
                    .inbox
                    .send(RouterMessage::Navigate("/post/".to_string()));
            }
        });
    }
}

fn edit_message(request: &mut Request<AppState>) -> impl Route<AppState> {
    |ui: &mut Ui, state: &mut AppState| {
        background(ui, ui.style().visuals.window_fill, |ui| {
            ui.heading("Edit Message");
            ui.text_edit_singleline(&mut state.message);

            if ui.button("Save").clicked() {
                state.inbox.send(RouterMessage::Back);
            }
        });
    }
}

fn post(request: &mut Request<AppState>) -> impl Route<AppState> {
    let id = request.params.get("id").map(ToOwned::to_owned);

    move |ui: &mut Ui, state: &mut AppState| {
        background(ui, ui.style().visuals.extreme_bg_color, |ui| {
            if let Some(id) = &id {
                ui.label(format!("Post: {}", id));

                ui.label(
                    "Hello these are a couple words to get some text to pretend there is content",
                );
            } else {
                ui.label("Post not found");
            }

            if ui.button("back").clicked() {
                state.inbox.send(RouterMessage::Back);
            }
        });
    }
}

fn background(ui: &mut Ui, color: Color32, content: impl FnOnce(&mut Ui)) {
    Frame::none().fill(color).inner_margin(16.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());
        content(ui);
    });
}

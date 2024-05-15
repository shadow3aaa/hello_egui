mod transition;

use crate::transition::{ActiveTransition, ActiveTransitionResult, Transition, TransitionType};
use egui::emath::ease_in_ease_out;
use egui::Ui;

pub trait Handler<State> {
    fn handle(&mut self, state: &mut Request<State>) -> Box<dyn Route<State>>;
}

pub trait Route<State> {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State);
}

struct RouteState<State> {
    route: Box<dyn Route<State>>,
}

#[derive(Debug, Clone)]
pub struct TransitionConfig {
    duration: Option<f32>,
    easing: fn(f32) -> f32,
    _in: Transition,
    out: Transition,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            duration: None,
            easing: ease_in_ease_out,
            _in: transition::SlideTransition::new(1.0).into(),
            out: transition::SlideTransition::new(-0.1).into(),
        }
    }
}

impl TransitionConfig {
    pub fn new(_in: impl Into<Transition>, out: impl Into<Transition>) -> Self {
        Self {
            _in: _in.into(),
            out: out.into(),
            ..Self::default()
        }
    }

    pub fn slide() -> Self {
        Self::default()
    }

    pub fn fade() -> Self {
        Self::new(transition::FadeTransition, transition::FadeTransition)
    }

    pub fn none() -> Self {
        Self::new(transition::NoTransition, transition::NoTransition)
    }

    pub fn with_easing(mut self, easing: fn(f32) -> f32) -> Self {
        self.easing = easing;
        self
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }
}

pub struct EguiRouter<State> {
    router: matchit::Router<Box<dyn Handler<State>>>,
    pub state: State,
    history: Vec<RouteState<State>>,

    forward_transition: TransitionConfig,
    backward_transition: TransitionConfig,

    current_transition: Option<ActiveTransition>,
    default_duration: Option<f32>,
}

pub struct Request<'a, State = ()> {
    pub params: matchit::Params<'a, 'a>,
    pub state: &'a mut State,
}

impl<State> EguiRouter<State> {
    pub fn new(state: State) -> Self {
        Self {
            router: matchit::Router::new(),
            state,
            history: Vec::new(),
            // default_transition: transition::Transition::Fade(transition::FadeTransition),
            current_transition: None,
            forward_transition: TransitionConfig::default(),
            backward_transition: TransitionConfig::default(),
            default_duration: None,
        }
    }

    pub fn with_transition(mut self, transition: TransitionConfig) -> Self {
        self.forward_transition = transition.clone();
        self.backward_transition = transition;
        self
    }

    pub fn with_forward_transition(mut self, transition: TransitionConfig) -> Self {
        self.forward_transition = transition;
        self
    }

    pub fn with_backward_transition(mut self, transition: TransitionConfig) -> Self {
        self.backward_transition = transition;
        self
    }

    pub fn with_default_duration(mut self, duration: f32) -> Self {
        self.default_duration = Some(duration);
        self
    }

    pub fn route(&mut self, route: impl Into<String>, handler: impl Handler<State> + 'static) {
        self.router
            .insert(route.into(), Box::new(handler))
            .expect("Invalid route");
    }

    pub fn navigate_transition(
        &mut self,
        route: impl Into<String>,
        transition_config: TransitionConfig,
    ) {
        let route = route.into();
        let mut handler = self.router.at_mut(&route);

        if let Ok(handler) = handler {
            let route = handler.value.handle(&mut Request {
                state: &mut self.state,
                params: handler.params,
            });
            self.history.push(RouteState { route });

            self.current_transition = Some(
                ActiveTransition::forward(transition_config)
                    .with_default_duration(self.default_duration),
            );
        } else {
            eprintln!("Failed to navigate to route: {}", route);
        }
    }

    pub fn back_transition(&mut self, transition_config: TransitionConfig) {
        if self.history.len() > 1 {
            self.current_transition = Some(
                ActiveTransition::backward(transition_config)
                    .with_default_duration(self.default_duration),
            );
        }
    }

    pub fn navigate(&mut self, route: impl Into<String>) {
        self.navigate_transition(route, self.forward_transition.clone());
    }

    pub fn back(&mut self) {
        self.back_transition(self.backward_transition.clone());
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if let Some((last, previous)) = self.history.split_last_mut() {
            let result = if let Some(transition) = &mut self.current_transition {
                Some(transition.show(
                    ui,
                    &mut self.state,
                    |ui, state| {
                        last.route.ui(ui, state);
                    },
                    previous.last_mut().map(|r| {
                        |ui: &mut _, state: &mut _| {
                            r.route.ui(ui, state);
                        }
                    }),
                ))
            } else {
                last.route.ui(ui, &mut self.state);
                None
            };

            match result {
                Some(ActiveTransitionResult::Done) => {
                    self.current_transition = None;
                }
                Some(ActiveTransitionResult::DonePop) => {
                    self.current_transition = None;
                    self.history.pop();
                }
                Some(ActiveTransitionResult::Continue) | None => {}
            }
        }
    }
}

impl<F, State, R: Route<State> + 'static> Handler<State> for F
where
    F: Fn(&mut Request<State>) -> R,
{
    fn handle(&mut self, request: &mut Request<State>) -> Box<dyn Route<State>> {
        Box::new(self(request))
    }
}

// impl<F, Fut, State, R: 'static> Handler<State> for F
// where
//     F: Fn(&mut State) -> Fut,
//     Fut: std::future::Future<Output = R>,
// {
//     async fn handle(&mut self, state: &mut State) -> Box<dyn Route<State>> {
//         Box::new((self(state)).await)
//     }
// }

impl<F: FnMut(&mut Ui, &mut State), State> Route<State> for F {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        self(ui, state)
    }
}

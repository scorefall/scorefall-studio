use fon::stereo::Stereo32;
use fon::{Audio, Sink};
use pasts::{block_on, wait};
use semtext::input::Action;
use semtext::text::{Color, Corner, Intensity, Outline, Stroke, Theme};
use semtext::widget::{BorderStyle, Label};
use semtext::{grid_area, Error, Screen};
use wavy::{Microphone, MicrophoneStream, Speakers, SpeakersSink};

/// An event handled by the event loop.
enum Event<'a> {
    /// Speaker is ready to play more audio.
    Play(SpeakersSink<'a, Stereo32>),
    /// Microphone has recorded some audio.
    Record(MicrophoneStream<'a, Stereo32>),
    /// User Interface event.
    Action(Result<Action, Error>),
}

/// Shared state between tasks on the thread.
struct State {
    /// Temporary buffer for holding real-time audio samples.
    buffer: Audio<Stereo32>,
}

impl State {
    /// Event loop.
    fn event(&mut self, event: Event<'_>) -> bool {
        match event {
            Event::Play(mut speakers) => speakers.stream(self.buffer.drain()),
            Event::Record(microphone) => self.buffer.extend(microphone),
            Event::Action(action) => return self.action(action.unwrap()),
        }
        true
    }

    /// Action handler.
    fn action(&mut self, action: Action) -> bool {
        match action {
            Action::Quit() => false,
            _ => true,
        }
    }
}

/// Program start.
fn main() {
    let mut state = State {
        buffer: Audio::with_silence(48_000, 0),
    };
    let mut speakers = Speakers::default();
    let mut microphone = Microphone::default();
    let mut screen = Screen::new().unwrap();
    screen.set_title("ScoreFall Studio").unwrap();
    screen.set_theme(Theme {
        background: Color::Black(Intensity::Normal),
        foreground: Color::White(Intensity::Normal),
        primary: Color::White(Intensity::Bright),
        focused: Color::Blue(Intensity::Normal),
        interacting: Color::Green(Intensity::Normal),
        dark_shadow: Color::Black(Intensity::Bright),
        light_shadow: Color::White(Intensity::Normal),
        enabled_border: BorderStyle::Simple(Outline::Light(Stroke::Solid, Corner::Rounded)),
        disabled_border: BorderStyle::Simple(Outline::Light(Stroke::Solid, Corner::Rounded)),
        button_released_border: BorderStyle::Simple(Outline::Light(Stroke::Solid, Corner::Rounded)),
        button_pressed_border: BorderStyle::Simple(Outline::Light(Stroke::Solid, Corner::Rounded)),
    });

    block_on(async move {
        let a = Label::new("ScoreFall Studio");
        let grid = grid_area!([a]).unwrap();

        while state.event(wait! {
            Event::Record(microphone.record().await),
            Event::Play(speakers.play().await),
            Event::Action(screen.step(&grid).await),
        }) {}
    });
}

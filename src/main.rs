use fon::chan::Ch32;
use fon::mono::Mono32;
use fon::stereo::Stereo32;
use fon::{Audio, Frame, Sink};
use pasts::{block_on, wait};
use semtext::input::Action;
use semtext::layout::{Cells, Pos};
use semtext::text::{Color, Corner, Intensity, Outline, Stroke, Theme, TextStyle};
use semtext::widget::{BorderStyle, Label};
use semtext::{grid_area, Error, Screen, Widget};
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

/// One of the tracks of the song.
enum Track {
    /// Track is Mono
    Mono(Audio<Mono32>),
    /// Track is Stereo
    Stereo(Audio<Stereo32>),
}

/// Shared state between tasks on the thread.
struct State {
    /// Master gain
    gain: Ch32,
}

impl State {
    /// Event loop.
    fn event(&mut self, event: Event<'_>) -> bool {
        match event {
            Event::Play(mut speakers) => {} /*speakers.stream(self.buffer.drain())*/
            Event::Record(_microphone) => {} /*self.buffer.extend(microphone)*/
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

/// UI Widget for Displaying Audio
struct AudioTracks {
    // How many samples per text character there are when rendering
    zoom: f32,
    // The audio tracks (must have the same Hz).
    tracks: Vec<Track>,
    // The length of the longest audio track in `tracks`
    len: usize,
    // The total number of channels out of all the `tracks`
    channels: usize,
}

impl AudioTracks {
    fn draw_sample(&self, cells: &mut Cells<'_>, col: u16, row: i32, sample: f32) -> Result<(), Error> {
        let mut blocks = (sample * 4.0 * 8.0).round() as i32;
        if blocks > 0 {
            cells.set_style(
                TextStyle::default()
                    .with_background(Color::Black(Intensity::Normal))
                    .with_foreground(Color::Blue(Intensity::Bright))
            )?;
        
            let mut i = 0;
            while blocks >= 8 {
                cells.move_to(col, (row + 3 - i) as u16)?;
                cells.print_char('█')?;
                blocks -= 8;
                i += 1;
            }
            let symbol = match blocks % 8 {
                0 => None,
                1 => Some('▁'),
                2 => Some('▂'),
                3 => Some('▃'),
                4 => Some('▄'),
                5 => Some('▅'),
                6 => Some('▆'),
                7 => Some('▇'),
                _ => unreachable!(),
            };
            if let Some(symbol) = symbol {
                cells.move_to(col, (row + 3 - i) as u16)?;
                cells.print_char(symbol)?;
            }
        } else if blocks < 0 {
            cells.set_style(
                TextStyle::default()
                    .with_foreground(Color::Black(Intensity::Normal))
                    .with_background(Color::Blue(Intensity::Bright))
            )?;
        
            let mut i = 0;
            while blocks <= -8 {
                cells.move_to(col, (row + 4 + i) as u16)?;
                cells.print_char(' ')?;
                blocks += 8;
                i += 1;
            }
            let symbol = match (-blocks) % 8 {
                0 => None,
                1 => Some('▇'),
                2 => Some('▆'),
                3 => Some('▅'),
                4 => Some('▄'),
                5 => Some('▃'),
                6 => Some('▂'),
                7 => Some('▁'),
                _ => unreachable!(),
            };
            if let Some(symbol) = symbol {
                cells.move_to(col, (row + 4 + i) as u16)?;
                cells.print_char(symbol)?;
            }
        }
        
        cells.set_style(
            TextStyle::default()
                .with_background(Color::Black(Intensity::Normal))
                .with_foreground(Color::White(Intensity::Bright))
        )?;
        
        Ok(())
    }
}

impl Widget for AudioTracks {
    /*fn bounds(&self, theme: &Theme) -> () {
        let row = ((self.channels * 8) + (self.channels - 1)) as u16;
        let col = (self.len as f32 * self.zoom).ceil() as u16;

/*        ()::default()
            .with_columns(col..col + 1)
            .with_rows(row..row + 1)*/
    }*/

    fn draw(&self, cells: &mut Cells<'_>, pos: Pos) -> Result<(), Error> {
        let mut row = 0i32;
        for track in &self.tracks {
            for col in 0..(self.len as f32 * self.zoom).ceil() as u16 {
                cells.move_to(col, row as u16)?;
                cells.print_char('━')?;
            }
            row += 1;
            for col in 0..(self.len as f32 * self.zoom).ceil() as u16 {
                match track {
                    Track::Mono(ref audio) => {
                        let sample: f32 = audio
                            .get((col as f32 * self.zoom).floor() as usize)
                            .unwrap()
                            .channels()[0]
                            .into();
                        self.draw_sample(cells, col, row, sample)?;
                    }
                    Track::Stereo(ref audio) => {
                        let left: f32 = audio
                            .get((col as f32 * self.zoom).floor() as usize)
                            .unwrap()
                            .channels()[0]
                            .into();
                        let right: f32 = audio
                            .get((col as f32 * self.zoom).floor() as usize)
                            .unwrap()
                            .channels()[1]
                            .into();
                        self.draw_sample(cells, col, row, left)?;
                        cells.move_to(col, (row + 8) as u16)?;
                        cells.print_char('─')?;
                        self.draw_sample(cells, col, row + 9, right)?;
                    }
                }
            }
            row += match track {
                Track::Mono(_) => 9,
                Track::Stereo(_) => 17,
            };
        }
        for col in 0..(self.len as f32 * self.zoom).ceil() as u16 {
            cells.move_to(col, row as u16)?;
            cells.print_char('━')?;
        }
        Ok(())
    }
}

/// Program start.
fn main() {
    let mut state = State {
        gain: Ch32::new(1.0),
        // buffer: Audio::with_silence(48_000, 0),
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
        button_border: BorderStyle::Simple(Outline::Light(Stroke::Solid, Corner::Rounded)),
        normal_border: BorderStyle::Simple(Outline::Light(Stroke::Solid, Corner::Rounded)),
    });

    block_on(async move {
        let a = Label::new("ScoreFall Studio");
        let tracks = AudioTracks {
            zoom: 1.0,
            tracks: vec![Track::Stereo(Audio::with_f32_buffer(
                48_000,
                [0.025, 1.0, -0.1, 0.9, 1.0, -1.0, 0.5, 0.5, -0.5, -0.5],
            ))],
            len: 5,
            channels: 1,
        };

        let grid = grid_area!([a][tracks]).unwrap();

        while state.event(wait! {
            Event::Record(microphone.record().await),
            Event::Play(speakers.play().await),
            Event::Action(screen.step(&grid).await),
        }) {}
    });
}

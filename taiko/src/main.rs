use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, Image, Mesh, Rect, Text};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, ContextBuilder, GameResult};

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const NOTE_SPEED: f32 = 400.0;
const HIT_LINE_X: f32 = 200.0;
const DRUM_WIDTH: f32 = 150.0;
const DRUM_HEIGHT: f32 = 150.0;

#[derive(Debug, Clone, Copy)]
enum NoteType {
    Don,    // Red rim (center hit)
    Kat,    // Blue rim (rim hit)
    BigDon, // Large red note
    BigKat, // Large blue note
}

impl NoteType {
    fn color(&self) -> Color {
        match self {
            NoteType::Don | NoteType::BigDon => Color::RED,
            NoteType::Kat | NoteType::BigKat => Color::BLUE,
        }
    }

    fn is_big(&self) -> bool {
        matches!(self, NoteType::BigDon | NoteType::BigKat)
    }

    fn size(&self) -> f32 {
        if self.is_big() {
            80.0
        } else {
            60.0
        }
    }

    fn keys(&self) -> (KeyCode, KeyCode) {
        match self {
            NoteType::Don | NoteType::BigDon => (KeyCode::D, KeyCode::F),
            NoteType::Kat | NoteType::BigKat => (KeyCode::J, KeyCode::K),
        }
    }
}

#[derive(Debug)]
struct Note {
    note_type: NoteType,
    x: f32,
    y: f32,
    hit: bool,
    accuracy: Option<Accuracy>,
}

#[derive(Debug, Clone)]
enum Accuracy {
    Miss,
    Hit50,
    Hit100,
    Hit300,
}

struct TaikoGame {
    notes: Vec<Note>,
    score: i32,
    combo: i32,
    last_spawn_time: f64,
    don_image: Image,
    kat_image: Image,
    big_don_image: Image,
    big_kat_image: Image,
    accuracy_display: Option<Accuracy>,
    accuracy_display_time: std::time::Instant,
}

impl TaikoGame {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let don_image = Image::from_path(ctx, "/taikohitcircle.png")?;
        let kat_image = Image::from_path(ctx, "/taikohitcircle.png")?; // Using same for now
        let big_don_image = Image::from_path(ctx, "/taikobigcircle.png")?;
        let big_kat_image = Image::from_path(ctx, "/taikobigcircle.png")?; // Using same for now
        
        Ok(Self {
            notes: Vec::new(),
            score: 0,
            combo: 0,
            last_spawn_time: 0.0,
            don_image,
            kat_image,
            big_don_image,
            big_kat_image,
            accuracy_display: None,
            accuracy_display_time: std::time::Instant::now(),
        })
    }

    fn spawn_note(&mut self, note_type: NoteType) {
        self.notes.push(Note {
            note_type,
            x: SCREEN_WIDTH + 50.0,
            y: SCREEN_HEIGHT / 2.0,
            hit: false,
            accuracy: None,
        });
    }

    fn check_hit(&mut self, key: KeyCode) {
        let hit_window = 50.0;
        
        for note in &mut self.notes {
            if note.hit {
                continue;
            }

            let (key1, key2) = note.note_type.keys();
            if key != key1 && key != key2 {
                continue;
            }

            let distance = (note.x - HIT_LINE_X).abs();
            if distance <= hit_window {
                let timing_diff = distance;
                let accuracy = if timing_diff <= 20.0 {
                    Accuracy::Hit300
                } else if timing_diff <= 50.0 {
                    Accuracy::Hit100
                } else if timing_diff <= 100.0 {
                    Accuracy::Hit50
                } else {
                    Accuracy::Miss
                };
                
                match accuracy {
                    Accuracy::Hit300 | Accuracy::Hit100 | Accuracy::Hit50 => {
                        self.score += match accuracy {
                            Accuracy::Hit300 => 300,
                            Accuracy::Hit100 => 100,
                            Accuracy::Hit50 => 50,
                            _ => 0,
                        };
                        self.combo += 1;
                    }
                    Accuracy::Miss => {
                        self.combo = 0;
                    }
                }
                
                note.accuracy = Some(accuracy.clone());
                note.hit = true;
                self.accuracy_display = Some(accuracy);
                self.accuracy_display_time = std::time::Instant::now();
                break;
            }
        }
    }
}

impl EventHandler for TaikoGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ctx.time.delta().as_secs_f64();
        
        // Update notes
        for note in &mut self.notes {
            if !note.hit {
                note.x -= NOTE_SPEED * dt as f32;
            }
        }

        // Remove notes that are off screen or hit
        self.notes.retain(|note| note.x > -100.0 && !note.hit);

        // Spawn new notes periodically
        self.last_spawn_time += dt;
        if self.last_spawn_time > 0.8 {
            let note_types = [
                NoteType::Don,
                NoteType::Kat,
                NoteType::BigDon,
                NoteType::BigKat,
            ];
            let random_note = note_types[rand::random::<usize>() % 4];
            self.spawn_note(random_note);
            self.last_spawn_time = 0.0;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::new(0.1, 0.1, 0.1, 1.0));

        // Draw drum (center hit area)
        let drum_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(
                HIT_LINE_X - DRUM_WIDTH / 2.0,
                SCREEN_HEIGHT / 2.0 - DRUM_HEIGHT / 2.0,
                DRUM_WIDTH,
                DRUM_HEIGHT,
            ),
            Color::new(0.3, 0.3, 0.3, 0.8),
        )?;
        canvas.draw(&drum_mesh, DrawParam::default());

        // Draw drum center (red area)
        let center_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(
                HIT_LINE_X - DRUM_WIDTH / 3.0,
                SCREEN_HEIGHT / 2.0 - DRUM_HEIGHT / 3.0,
                DRUM_WIDTH * 2.0 / 3.0,
                DRUM_HEIGHT * 2.0 / 3.0,
            ),
            Color::new(0.5, 0.2, 0.2, 0.5),
        )?;
        canvas.draw(&center_mesh, DrawParam::default());

        // Draw hit line
        let hit_line_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(HIT_LINE_X - 2.0, 100.0, 4.0, SCREEN_HEIGHT - 200.0),
            Color::WHITE,
        )?;
        canvas.draw(&hit_line_mesh, DrawParam::default());

        // Draw notes
        for note in &self.notes {
            if !note.hit {
                let (image, color_mod) = match note.note_type {
                    NoteType::Don => (&self.don_image, Color::RED),
                    NoteType::Kat => (&self.kat_image, Color::BLUE),
                    NoteType::BigDon => (&self.big_don_image, Color::RED),
                    NoteType::BigKat => (&self.big_kat_image, Color::BLUE),
                };
                
                let scale = if note.note_type.is_big() { 1.2 } else { 1.0 };
                
                canvas.draw(
                    image,
                    DrawParam::default()
                        .dest([note.x, note.y])
                        .scale([scale, scale])
                        .color(color_mod)
                        .offset([0.5, 0.5]), // Center the image
                );
                
                if note.note_type.is_big() {
                    // Draw border for big notes
                    let border_mesh = Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::stroke(3.0),
                        Rect::new(
                            note.x - note.note_type.size() / 2.0 - 3.0,
                            note.y - note.note_type.size() / 2.0 - 3.0,
                            note.note_type.size() + 6.0,
                            note.note_type.size() + 6.0,
                        ),
                        Color::WHITE,
                    )?;
                    canvas.draw(&border_mesh, DrawParam::default());
                }
            }
        }

        // Draw UI
        let score_text = Text::new(format!("Score: {}", self.score));
        canvas.draw(&score_text, DrawParam::default().dest([10.0, 10.0]));

        let combo_text = Text::new(format!("Combo: {}", self.combo));
        canvas.draw(&combo_text, DrawParam::default().dest([10.0, 40.0]));

        // Draw accuracy rating if recently hit
        if let Some(accuracy) = &self.accuracy_display {
            if self.accuracy_display_time.elapsed().as_secs_f32() < 1.0 {
                let accuracy_text = match accuracy {
                    Accuracy::Hit300 => "300!",
                    Accuracy::Hit100 => "100",
                    Accuracy::Hit50 => "50",
                    Accuracy::Miss => "MISS",
                };
                let accuracy_display = Text::new(accuracy_text);
                let color = match accuracy {
                    Accuracy::Hit300 => Color::new(1.0, 1.0, 0.0, 1.0), // Gold
                    Accuracy::Hit100 => Color::new(0.5, 0.5, 1.0, 1.0), // Blue
                    Accuracy::Hit50 => Color::new(0.5, 1.0, 0.5, 1.0), // Green
                    Accuracy::Miss => Color::new(1.0, 0.0, 0.0, 1.0), // Red
                };
                canvas.draw(&accuracy_display, DrawParam::default()
                    .dest([SCREEN_WIDTH / 2.0 - 50.0, SCREEN_HEIGHT / 2.0])
                    .color(color));
            }
        }

        let instructions = Text::new("D/F: Don (Red) | J/K: Kat (Blue) | ESC: Quit");
        canvas.draw(&instructions, DrawParam::default().dest([10.0, SCREEN_HEIGHT - 30.0]));

        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(KeyCode::D) | Some(KeyCode::F) | Some(KeyCode::J) | Some(KeyCode::K) => {
                if let Some(key) = input.keycode {
                    self.check_hit(key);
                }
            }
            Some(KeyCode::Escape) => ctx.request_quit(),
            _ => {}
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("taiko", "author")
        .window_setup(ggez::conf::WindowSetup::default().title("Taiko Rhythm Game"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()?;

    let game = TaikoGame::new(&mut ctx)?;
    event::run(ctx, event_loop, game)
}

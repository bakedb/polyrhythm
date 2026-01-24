use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, Mesh, Rect, Text};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, ContextBuilder, GameResult};

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const NOTE_SPEED: f32 = 300.0;
const HIT_LINE_Y: f32 = 500.0;
const LANE_WIDTH: f32 = 80.0;
const FRET_WIDTH: f32 = 60.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum FretColor {
    Green,  // D key
    Red,    // F key
    Yellow, // J key
    Blue,   // K key
    Orange, // L key (extra)
}

impl FretColor {
    fn key_code(&self) -> KeyCode {
        match self {
            FretColor::Green => KeyCode::D,
            FretColor::Red => KeyCode::F,
            FretColor::Yellow => KeyCode::J,
            FretColor::Blue => KeyCode::K,
            FretColor::Orange => KeyCode::L,
        }
    }

    fn lane_x(&self) -> f32 {
        match self {
            FretColor::Green => 200.0,
            FretColor::Red => 280.0,
            FretColor::Yellow => 360.0,
            FretColor::Blue => 440.0,
            FretColor::Orange => 520.0,
        }
    }

    fn color(&self) -> Color {
        match self {
            FretColor::Green => Color::GREEN,
            FretColor::Red => Color::RED,
            FretColor::Yellow => Color::YELLOW,
            FretColor::Blue => Color::BLUE,
            FretColor::Orange => Color::new(1.0, 0.5, 0.0, 1.0),
        }
    }
}

#[derive(Debug)]
enum NoteType {
    Single { color: FretColor },
    Hold { color: FretColor, duration: f32 },
}

struct Note {
    note_type: NoteType,
    y: f32,
    hit: bool,
    missed: bool,
    hold_start_hit: bool,
}

struct CloneHeroGame {
    notes: Vec<Note>,
    score: i32,
    combo: i32,
    last_spawn_time: f64,
    pressed_frets: [bool; 5],
}

impl CloneHeroGame {
    fn new() -> Self {
        Self {
            notes: Vec::new(),
            score: 0,
            combo: 0,
            last_spawn_time: 0.0,
            pressed_frets: [false; 5],
        }
    }

    fn spawn_note(&mut self, note_type: NoteType) {
        self.notes.push(Note {
            note_type,
            y: -50.0,
            hit: false,
            missed: false,
            hold_start_hit: false,
        });
    }

    fn check_hit(&mut self, fret: FretColor) {
        let hit_window = 50.0;
        
        for note in &mut self.notes {
            if note.hit || note.missed {
                continue;
            }

            let distance = (note.y - HIT_LINE_Y).abs();
            if distance < hit_window {
                match &note.note_type {
                    NoteType::Single { color } if *color == fret => {
                        note.hit = true;
                        self.combo += 1;
                        
                        if distance < 20.0 {
                            self.score += 100; // Perfect
                        } else if distance < 35.0 {
                            self.score += 50; // Good
                        } else {
                            self.score += 25; // OK
                        }
                        break;
                    }
                    NoteType::Hold { color, .. } if *color == fret && !note.hold_start_hit => {
                        note.hold_start_hit = true;
                        self.combo += 1;
                        self.score += 50;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    fn update_hold_notes(&mut self) {
        for note in &mut self.notes {
            if let NoteType::Hold { color, .. } = &note.note_type {
                if note.hold_start_hit && !note.hit {
                    let fret_index = match color {
                        FretColor::Green => 0,
                        FretColor::Red => 1,
                        FretColor::Yellow => 2,
                        FretColor::Blue => 3,
                        FretColor::Orange => 4,
                    };
                    
                    if !self.pressed_frets[fret_index] {
                        // Player released the hold note
                        note.hit = true;
                        self.combo = 0;
                    }
                }
            }
        }
    }
}

impl EventHandler for CloneHeroGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ctx.time.delta().as_secs_f64();
        
        // Update notes
        for note in &mut self.notes {
            if !note.hit && !note.missed {
                note.y += NOTE_SPEED * dt as f32;
                
                // Check if note missed the hit line
                if note.y > HIT_LINE_Y + 50.0 && !note.hit {
                    note.missed = true;
                    self.combo = 0;
                }
            }
        }

        // Update hold notes
        self.update_hold_notes();

        // Remove notes that are off screen or hit
        self.notes.retain(|note| note.y < SCREEN_HEIGHT + 50.0 && !note.hit);

        // Spawn new notes periodically
        self.last_spawn_time += dt;
        if self.last_spawn_time > 1.0 {
            let colors = [
                FretColor::Green,
                FretColor::Red,
                FretColor::Yellow,
                FretColor::Blue,
                FretColor::Orange,
            ];
            let random_color = colors[rand::random::<usize>() % 5];
            
            // Randomly choose between single and hold notes
            if rand::random::<f32>() < 0.7 {
                self.spawn_note(NoteType::Single { color: random_color });
            } else {
                let duration = 100.0 + rand::random::<f32>() * 200.0;
                self.spawn_note(NoteType::Hold { color: random_color, duration });
            }
            
            self.last_spawn_time = 0.0;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // Draw fretboard lanes
        for i in 0..5 {
            let x = 200.0 + i as f32 * LANE_WIDTH;
            let mesh = Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(x, 0.0, LANE_WIDTH, SCREEN_HEIGHT),
                Color::new(0.2, 0.2, 0.2, 0.3),
            )?;
            canvas.draw(&mesh, DrawParam::default());
        }

        // Draw fret buttons at bottom
        for (i, color) in [
            FretColor::Green,
            FretColor::Red,
            FretColor::Yellow,
            FretColor::Blue,
            FretColor::Orange,
        ].iter().enumerate() {
            let x = color.lane_x() + 10.0;
            let y = HIT_LINE_Y + 20.0;
            
            let mut fret_color = color.color();
            if self.pressed_frets[i] {
                fret_color = Color::new(
                    fret_color.r,
                    fret_color.g,
                    fret_color.b,
                    0.8,
                );
            } else {
                fret_color = Color::new(
                    fret_color.r * 0.5,
                    fret_color.g * 0.5,
                    fret_color.b * 0.5,
                    0.5,
                );
            }
            
            let fret_mesh = Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(x, y, FRET_WIDTH, FRET_WIDTH),
                fret_color,
            )?;
            canvas.draw(&fret_mesh, DrawParam::default());
        }

        // Draw hit line
        let hit_line_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(150.0, HIT_LINE_Y, 500.0, 5.0),
            Color::WHITE,
        )?;
        canvas.draw(&hit_line_mesh, DrawParam::default());

        // Draw notes
        for note in &self.notes {
            if !note.hit && !note.missed {
                match &note.note_type {
                    NoteType::Single { color } => {
                        let note_mesh = Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            Rect::new(
                                color.lane_x() + 10.0,
                                note.y,
                                FRET_WIDTH,
                                FRET_WIDTH,
                            ),
                            color.color(),
                        )?;
                        canvas.draw(&note_mesh, DrawParam::default());
                    }
                    NoteType::Hold { color, duration } => {
                        // Draw hold note start
                        let start_mesh = Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            Rect::new(
                                color.lane_x() + 10.0,
                                note.y,
                                FRET_WIDTH,
                                FRET_WIDTH,
                            ),
                            color.color(),
                        )?;
                        canvas.draw(&start_mesh, DrawParam::default());
                        
                        // Draw hold note trail
                        let trail_color = if note.hold_start_hit {
                            Color::new(color.color().r, color.color().g, color.color().b, 0.8)
                        } else {
                            Color::new(color.color().r, color.color().g, color.color().b, 0.4)
                        };
                        
                        let trail_mesh = Mesh::new_rectangle(
                            ctx,
                            graphics::DrawMode::fill(),
                            Rect::new(
                                color.lane_x() + 25.0,
                                note.y + FRET_WIDTH,
                                20.0,
                                *duration,
                            ),
                            trail_color,
                        )?;
                        canvas.draw(&trail_mesh, DrawParam::default());
                    }
                }
            }
        }

        // Draw UI
        let score_text = Text::new(format!("Score: {}", self.score));
        canvas.draw(&score_text, DrawParam::default().dest([10.0, 10.0]));

        let combo_text = Text::new(format!("Combo: {}", self.combo));
        canvas.draw(&combo_text, DrawParam::default().dest([10.0, 40.0]));

        let instructions = Text::new("D: Green | F: Red | J: Yellow | K: Blue | L: Orange | ESC: Quit");
        canvas.draw(&instructions, DrawParam::default().dest([10.0, SCREEN_HEIGHT - 30.0]));

        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(KeyCode::D) => {
                self.pressed_frets[0] = true;
                self.check_hit(FretColor::Green);
            }
            Some(KeyCode::F) => {
                self.pressed_frets[1] = true;
                self.check_hit(FretColor::Red);
            }
            Some(KeyCode::J) => {
                self.pressed_frets[2] = true;
                self.check_hit(FretColor::Yellow);
            }
            Some(KeyCode::K) => {
                self.pressed_frets[3] = true;
                self.check_hit(FretColor::Blue);
            }
            Some(KeyCode::L) => {
                self.pressed_frets[4] = true;
                self.check_hit(FretColor::Orange);
            }
            Some(KeyCode::Escape) => ctx.request_quit(),
            _ => {}
        }
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: ggez::input::keyboard::KeyInput) -> GameResult {
        if let Some(key) = input.keycode {
            match key {
                KeyCode::D => self.pressed_frets[0] = false,
                KeyCode::F => self.pressed_frets[1] = false,
                KeyCode::J => self.pressed_frets[2] = false,
                KeyCode::K => self.pressed_frets[3] = false,
                KeyCode::L => self.pressed_frets[4] = false,
                _ => {}
            }
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("clone_hero", "author")
        .window_setup(ggez::conf::WindowSetup::default().title("Clone Hero Rhythm Game"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()?;

    let game = CloneHeroGame::new();
    event::run(ctx, event_loop, game)
}

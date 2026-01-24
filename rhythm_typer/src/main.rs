use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, Mesh, Rect, Text};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, ContextBuilder, GameResult};
use std::time::Instant;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const WORD_FALL_SPEED: f32 = 100.0;
const HIT_LINE_Y: f32 = 500.0;
const HIT_WINDOW: f32 = 50.0;

#[derive(Debug)]
struct Word {
    text: String,
    x: f32,
    y: f32,
    typed_chars: usize,
    completed: bool,
    accuracy: Option<Accuracy>,
    missed: bool,
    spawn_time: Instant,
}

#[derive(Debug, Clone)]
enum Accuracy {
    Miss,
    Hit50,
    Hit100,
    Hit150,
    Hit200,
    Hit300,
    Hit320,
}

impl Word {
    fn new(text: String, x: f32) -> Self {
        Self {
            text,
            x,
            y: -50.0,
            typed_chars: 0,
            completed: false,
            accuracy: None,
            missed: false,
            spawn_time: Instant::now(),
        }
    }

    fn is_fully_typed(&self) -> bool {
        self.typed_chars >= self.text.len()
    }

    fn get_display_text(&self) -> String {
        format!(
            "{}{}",
            &self.text[..self.typed_chars],
            &self.text[self.typed_chars..]
        )
    }

    fn check_char(&mut self, c: char) -> bool {
        if self.completed || self.missed || self.typed_chars >= self.text.len() {
            return false;
        }

        if self.text.chars().nth(self.typed_chars) == Some(c) {
            self.typed_chars += 1;
            if self.is_fully_typed() {
                self.completed = true;
            }
            true
        } else {
            false
        }
    }

    fn is_expired(&self) -> bool {
        self.y > HIT_LINE_Y + HIT_WINDOW || self.missed
    }
}

struct RhythmTyperGame {
    words: Vec<Word>,
    score: i32,
    combo: i32,
    last_spawn_time: f64,
    accuracy_display: Option<Accuracy>,
    accuracy_display_time: std::time::Instant,
    current_input: String,
    word_pool: Vec<String>,
}

impl RhythmTyperGame {
    fn new() -> Self {
        let word_pool = vec![
            "rhythm".to_string(),
            "beat".to_string(),
            "music".to_string(),
            "tempo".to_string(),
            "dance".to_string(),
            "groove".to_string(),
            "melody".to_string(),
            "harmony".to_string(),
            "timing".to_string(),
            "sync".to_string(),
            "flow".to_string(),
        ];
        
        Self {
            words: Vec::new(),
            score: 0,
            combo: 0,
            last_spawn_time: 0.0,
            accuracy_display: None,
            accuracy_display_time: std::time::Instant::now(),
            current_input: String::new(),
            word_pool,
        }
    }

    fn spawn_word(&mut self) {
        let word_text = self.word_pool[rand::random::<usize>() % self.word_pool.len()].clone();
        let x = 200.0 + rand::random::<f32>() * 400.0;
        self.words.push(Word::new(word_text, x));
    }

    fn handle_char_input(&mut self, c: char) {
        let mut hit_any = false;
        
        for word in &mut self.words {
            if word.check_char(c) {
                hit_any = true;
                if word.hit {
                    self.combo += 1;
                    let base_score = word.text.len() as i32 * 10;
                    let combo_bonus = (self.combo / 5) * 5;
                    self.score += base_score + combo_bonus;
                }
                break;
            }
        }

        if !hit_any {
            self.combo = 0;
        }
    }
}

impl EventHandler for RhythmTyperGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ctx.time.delta().as_secs_f32();
        
        // Update word positions
        for word in &mut self.words {
            if !word.hit && !word.missed {
                word.y += WORD_FALL_SPEED * dt;
                
                // Check if word missed the hit line
                if word.y > HIT_LINE_Y + HIT_WINDOW && !word.hit {
                    word.missed = true;
                    self.combo = 0;
                }
            }
        }

        // Remove expired words
        self.words.retain(|w| !w.is_expired());

        // Spawn new words periodically
        if self.last_spawn_time.elapsed().as_secs_f32() > 2.0 {
            self.spawn_word();
            self.last_spawn_time = Instant::now();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::new(0.05, 0.05, 0.1, 1.0));

        // Draw hit line
        let hit_line_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(0.0, HIT_LINE_Y, SCREEN_WIDTH, 3.0),
            Color::YELLOW,
        )?;
        canvas.draw(&hit_line_mesh, DrawParam::default());

        // Draw words
        for word in &self.words {
            if !word.hit {
                let display_text = word.get_display_text();
                
                // Draw typed part in green
                if word.typed_chars > 0 {
                    let typed_text = Text::new(&word.text[..word.typed_chars]);
                    canvas.draw(
                        &typed_text,
                        DrawParam::default()
                            .dest([word.x, word.y])
                            .color(Color::GREEN),
                    );
                }
                
                // Draw untyped part in white
                if word.typed_chars < word.text.len() {
                    let untyped_text = Text::new(&word.text[word.typed_chars..]);
                    let typed_width = if word.typed_chars > 0 {
                        let typed_text = Text::new(&word.text[..word.typed_chars]);
                        typed_text.measure(ctx).unwrap().x
                    } else {
                        0.0
                    };
                    
                    canvas.draw(
                        &untyped_text,
                        DrawParam::default()
                            .dest([word.x + typed_width, word.y])
                            .color(Color::WHITE),
                    );
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
                    Accuracy::Hit320 => "320!",
                    Accuracy::Hit300 => "300",
                    Accuracy::Hit200 => "200",
                    Accuracy::Hit150 => "150",
                    Accuracy::Hit100 => "100",
                    Accuracy::Hit50 => "50",
                    Accuracy::Miss => "MISS",
                };
                let accuracy_display = Text::new(accuracy_text);
                let color = match accuracy {
                    Accuracy::Hit320 | Accuracy::Hit300 => Color::new(1.0, 1.0, 0.0, 1.0), // Gold
                    Accuracy::Hit200 | Accuracy::Hit150 => Color::new(0.8, 0.6, 0.2, 1.0), // Orange
                    Accuracy::Hit100 => Color::new(0.5, 0.5, 1.0, 1.0), // Blue
                    Accuracy::Hit50 => Color::new(0.5, 1.0, 0.5, 1.0), // Green
                    Accuracy::Miss => Color::new(1.0, 0.0, 0.0, 1.0), // Red
                };
                canvas.draw(&accuracy_display, DrawParam::default()
                    .dest([SCREEN_WIDTH / 2.0 - 50.0, SCREEN_HEIGHT / 2.0])
                    .color(color));
            }
        }

        let input_text = Text::new("Type falling words! (DFJK are main rhythm keys)");
        canvas.draw(&input_text, DrawParam::default().dest([10.0, SCREEN_HEIGHT - 30.0]));

        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(KeyCode::Escape) => ctx.request_quit(),
            Some(KeyCode::Back) => {
                // Handle backspace if needed
                self.current_input.clear();
            }
            Some(key) => {
                // Try to get the character from the key
                if let Some(char) = self.get_char_from_key(key) {
                    self.handle_char_input(char);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl RhythmTyperGame {
    fn get_char_from_key(&self, key: KeyCode) -> Option<char> {
        match key {
            // Main DFJK keys for rhythm gameplay
            KeyCode::D => Some('d'),
            KeyCode::F => Some('f'),
            KeyCode::J => Some('j'),
            KeyCode::K => Some('k'),
            // Additional letters for word typing
            KeyCode::A => Some('a'),
            KeyCode::B => Some('b'),
            KeyCode::C => Some('c'),
            KeyCode::E => Some('e'),
            KeyCode::G => Some('g'),
            KeyCode::H => Some('h'),
            KeyCode::I => Some('i'),
            KeyCode::L => Some('l'),
            KeyCode::M => Some('m'),
            KeyCode::N => Some('n'),
            KeyCode::O => Some('o'),
            KeyCode::P => Some('p'),
            KeyCode::Q => Some('q'),
            KeyCode::R => Some('r'),
            KeyCode::S => Some('s'),
            KeyCode::T => Some('t'),
            KeyCode::U => Some('u'),
            KeyCode::V => Some('v'),
            KeyCode::W => Some('w'),
            KeyCode::X => Some('x'),
            KeyCode::Y => Some('y'),
            KeyCode::Z => Some('z'),
            _ => None,
        }
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("rhythm_typer", "author")
        .window_setup(ggez::conf::WindowSetup::default().title("Rhythm Typer"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()?;

    let game = RhythmTyperGame::new();
    event::run(ctx, event_loop, game)
}

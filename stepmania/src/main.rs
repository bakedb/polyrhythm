use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, Image, Mesh, Rect, Text};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, ContextBuilder, GameResult};

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const LANE_WIDTH: f32 = 100.0;
const ARROW_SPEED: f32 = 300.0;
const HIT_LINE_Y: f32 = 500.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ArrowDirection {
    Left,   // D key
    Down,   // F key  
    Up,     // J key
    Right,  // K key
}

impl ArrowDirection {
    fn key_code(&self) -> KeyCode {
        match self {
            ArrowDirection::Left => KeyCode::D,
            ArrowDirection::Down => KeyCode::F,
            ArrowDirection::Up => KeyCode::J,
            ArrowDirection::Right => KeyCode::K,
        }
    }

    fn lane_x(&self) -> f32 {
        match self {
            ArrowDirection::Left => 200.0,
            ArrowDirection::Down => 300.0,
            ArrowDirection::Up => 400.0,
            ArrowDirection::Right => 500.0,
        }
    }

    fn color(&self) -> Color {
        match self {
            ArrowDirection::Left => Color::RED,
            ArrowDirection::Down => Color::BLUE,
            ArrowDirection::Up => Color::GREEN,
            ArrowDirection::Right => Color::YELLOW,
        }
    }
}

#[derive(Debug)]
struct Arrow {
    direction: ArrowDirection,
    y: f32,
    hit: bool,
    accuracy: Option<Accuracy>,
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

struct StepmaniaGame {
    arrows: Vec<Arrow>,
    score: i32,
    combo: i32,
    last_spawn_time: f64,
    note_images: [Image; 4],
    accuracy_display: Option<Accuracy>,
    accuracy_display_time: std::time::Instant,
}

impl StepmaniaGame {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let note_images = [
            Image::from_path(ctx, "/mania-note1.png")?,  // Left/D
            Image::from_path(ctx, "/mania-note2.png")?,  // Down/F
            Image::from_path(ctx, "/mania-noteS.png")?,  // Up/J
            Image::from_path(ctx, "/mania-note1H.png")?, // Right/K
        ];
        
        Ok(Self {
            arrows: Vec::new(),
            score: 0,
            combo: 0,
            last_spawn_time: 0.0,
            note_images,
            accuracy_display: None,
            accuracy_display_time: std::time::Instant::now(),
        })
    }

    fn spawn_arrow(&mut self, direction: ArrowDirection) {
        self.arrows.push(Arrow {
            direction,
            y: -50.0,
            hit: false,
            accuracy: None,
        });
    }

    fn check_hit(&mut self, _ctx: &Context, direction: ArrowDirection) {
        let hit_window = 50.0;
        
        for arrow in &mut self.arrows {
            if arrow.hit || arrow.direction != direction {
                continue;
            }

            let distance = (arrow.y - HIT_LINE_Y).abs();
            if distance <= hit_window {
                let timing_diff = distance;
                let accuracy = if timing_diff <= 10.0 {
                    Accuracy::Hit320
                } else if timing_diff <= 20.0 {
                    Accuracy::Hit300
                } else if timing_diff <= 40.0 {
                    Accuracy::Hit200
                } else if timing_diff <= 60.0 {
                    Accuracy::Hit150
                } else if timing_diff <= 80.0 {
                    Accuracy::Hit100
                } else if timing_diff <= 100.0 {
                    Accuracy::Hit50
                } else {
                    Accuracy::Miss
                };
                
                match accuracy {
                    Accuracy::Hit320 | Accuracy::Hit300 | Accuracy::Hit200 | Accuracy::Hit150 | Accuracy::Hit100 | Accuracy::Hit50 => {
                        self.score += match accuracy {
                            Accuracy::Hit320 => 320,
                            Accuracy::Hit300 => 300,
                            Accuracy::Hit200 => 200,
                            Accuracy::Hit150 => 150,
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
                
                arrow.accuracy = Some(accuracy.clone());
                arrow.hit = true;
                self.accuracy_display = Some(accuracy);
                self.accuracy_display_time = std::time::Instant::now();
                break;
            }
        }
    }
}

impl EventHandler for StepmaniaGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ctx.time.delta().as_secs_f64();
        
        // Update arrows
        for arrow in &mut self.arrows {
            if !arrow.hit {
                arrow.y += ARROW_SPEED * dt as f32;
            }
        }

        // Remove arrows that are off screen or hit
        self.arrows.retain(|arrow| arrow.y < SCREEN_HEIGHT + 50.0 && !arrow.hit);

        // Spawn new arrows periodically
        self.last_spawn_time += dt;
        if self.last_spawn_time > 1.0 {
            let directions = [
                ArrowDirection::Left,
                ArrowDirection::Down,
                ArrowDirection::Up,
                ArrowDirection::Right,
            ];
            let random_dir = directions[rand::random::<usize>() % 4];
            self.spawn_arrow(random_dir);
            self.last_spawn_time = 0.0;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // Draw lanes
        for i in 0..4 {
            let x = 200.0 + i as f32 * LANE_WIDTH;
            let mesh = Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(x, 0.0, LANE_WIDTH, SCREEN_HEIGHT),
                Color::new(0.2, 0.2, 0.2, 0.5),
            )?;
            canvas.draw(&mesh, DrawParam::default());
        }

        // Draw hit line
        let hit_line_mesh = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            Rect::new(150.0, HIT_LINE_Y, 500.0, 5.0),
            Color::WHITE,
        )?;
        canvas.draw(&hit_line_mesh, DrawParam::default());

        // Draw arrows
        for arrow in &self.arrows {
            if !arrow.hit {
                let image_index = match arrow.direction {
                    ArrowDirection::Left => 0,
                    ArrowDirection::Down => 1,
                    ArrowDirection::Up => 2,
                    ArrowDirection::Right => 3,
                };
                
                canvas.draw(
                    &self.note_images[image_index],
                    DrawParam::default()
                        .dest([arrow.direction.lane_x() + 40.0, arrow.y])
                        .scale([0.4, 0.4]) // Much smaller scale to fix size issue
                        .offset([0.5, 0.5]), // Center the image
                );
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

        let instructions = Text::new("D: Left | F: Down | J: Up | K: Right | ESC: Quit");
        canvas.draw(&instructions, DrawParam::default().dest([10.0, SCREEN_HEIGHT - 30.0]));

        canvas.finish(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(KeyCode::D) => self.check_hit(ctx, ArrowDirection::Left),
            Some(KeyCode::F) => self.check_hit(ctx, ArrowDirection::Down),
            Some(KeyCode::J) => self.check_hit(ctx, ArrowDirection::Up),
            Some(KeyCode::K) => self.check_hit(ctx, ArrowDirection::Right),
            Some(KeyCode::Escape) => ctx.request_quit(),
            _ => {}
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("stepmania", "author")
        .window_setup(ggez::conf::WindowSetup::default().title("Stepmania Rhythm Game"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()?;

    let game = StepmaniaGame::new(&mut ctx)?;
    event::run(ctx, event_loop, game)
}

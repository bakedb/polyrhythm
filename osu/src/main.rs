use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, Image, Text};
use ggez::input::mouse::MouseButton;
use ggez::{Context, ContextBuilder, GameResult};
use std::time::Instant;

const SCREEN_WIDTH: f32 = 800.0;
const SCREEN_HEIGHT: f32 = 600.0;
const HIT_CIRCLE_RADIUS: f32 = 64.0;
const APPROACH_CIRCLE_MAX_RADIUS: f32 = 150.0;
const APPROACH_TIME_MS: u64 = 1000; // Time for approach circle to shrink
const HIT_WINDOW_MS: u64 = 200; // Hit timing window

#[derive(Debug)]
struct HitCircle {
    x: f32,
    y: f32,
    radius: f32,
    spawn_time: Instant,
    hit: bool,
    missed: bool,
    accuracy: Option<Accuracy>,
}

#[derive(Debug, Clone)]
enum Accuracy {
    Miss,
    Hit50,
    Hit100,
    Hit300,
}

impl HitCircle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            radius: HIT_CIRCLE_RADIUS,
            spawn_time: Instant::now(),
            hit: false,
            missed: false,
            accuracy: None,
        }
    }

    fn approach_circle_radius(&self) -> f32 {
        let elapsed = self.spawn_time.elapsed().as_millis() as f32;
        let progress = (elapsed / APPROACH_TIME_MS as f32).min(1.0);
        APPROACH_CIRCLE_MAX_RADIUS * (1.0 - progress) + self.radius * progress
    }

    fn is_expired(&self) -> bool {
        // Circle expires after approach time + hit window
        self.spawn_time.elapsed().as_millis() > (APPROACH_TIME_MS + HIT_WINDOW_MS * 2) as u128
    }

    fn check_click(&self, mouse_x: f32, mouse_y: f32) -> Option<Accuracy> {
        if self.hit || self.missed {
            return None;
        }

        let elapsed = self.spawn_time.elapsed().as_millis();
        if elapsed > (APPROACH_TIME_MS + HIT_WINDOW_MS * 2) as u128 {
            return None;
        }

        let distance = ((self.x - mouse_x).powi(2) + (self.y - mouse_y).powi(2)).sqrt();
        if distance <= self.radius {
            let timing_diff = (elapsed as i64 - APPROACH_TIME_MS as i64).abs();
            
            if timing_diff <= 50 {
                Some(Accuracy::Hit300)
            } else if timing_diff <= 100 {
                Some(Accuracy::Hit100)
            } else if timing_diff <= 150 {
                Some(Accuracy::Hit50)
            } else {
                Some(Accuracy::Miss)
            }
        } else {
            None
        }
    }
}

struct OsuGame {
    circles: Vec<HitCircle>,
    score: i32,
    combo: i32,
    last_spawn_time: Instant,
    hit_circle_image: Image,
    approach_circle_image: Image,
    accuracy_display: Option<Accuracy>,
    accuracy_display_time: Instant,
}

impl OsuGame {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        let hit_circle_image = Image::from_path(ctx, "/hitcircle.png")?;
        let approach_circle_image = Image::from_path(ctx, "/approachcircle.png")?;
        
        Ok(Self {
            circles: Vec::new(),
            score: 0,
            combo: 0,
            last_spawn_time: Instant::now(),
            hit_circle_image,
            approach_circle_image,
            accuracy_display: None,
            accuracy_display_time: Instant::now(),
        })
    }

    fn spawn_circle(&mut self) {
        let margin = HIT_CIRCLE_RADIUS + 20.0;
        let x = margin + rand::random::<f32>() * (SCREEN_WIDTH - 2.0 * margin);
        let y = margin + rand::random::<f32>() * (SCREEN_HEIGHT - 2.0 * margin);
        self.circles.push(HitCircle::new(x, y));
    }

    fn handle_click(&mut self, mouse_x: f32, mouse_y: f32) {
        for circle in &mut self.circles {
            if let Some(accuracy) = circle.check_click(mouse_x, mouse_y) {
                circle.accuracy = Some(accuracy.clone());
                match accuracy {
                    Accuracy::Hit300 => {
                        self.score += 300;
                        self.combo += 1;
                    }
                    Accuracy::Hit100 => {
                        self.score += 100;
                        self.combo += 1;
                    }
                    Accuracy::Hit50 => {
                        self.score += 50;
                        self.combo += 1;
                    }
                    Accuracy::Miss => {
                        self.combo = 0;
                    }
                }
                break;
            }
        }
    }
}

impl EventHandler for OsuGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Remove expired circles
        self.circles.retain(|c| !c.is_expired() && !c.hit);

        // Check for missed circles
        for circle in &mut self.circles {
            if !circle.hit && !circle.missed && circle.spawn_time.elapsed().as_millis() > (APPROACH_TIME_MS + HIT_WINDOW_MS) as u128 {
                circle.missed = true;
                self.combo = 0;
            }
        }

        // Spawn new circles periodically
        if self.last_spawn_time.elapsed().as_secs_f32() > 1.5 {
            self.spawn_circle();
            self.last_spawn_time = Instant::now();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::new(0.1, 0.1, 0.2, 1.0));

        // Draw hit circles
        for circle in &self.circles {
            if !circle.hit {
                // Draw approach circle
                let approach_radius = circle.approach_circle_radius();
                let scale = approach_radius / 64.0; // Base size of approach circle image
                canvas.draw(
                    &self.approach_circle_image,
                    DrawParam::default()
                        .dest([circle.x, circle.y])
                        .scale([scale, scale])
                        .offset([0.5, 0.5]), // Center the image
                );

                // Draw hit circle
                let scale = circle.radius / 64.0; // Base size of hit circle image
                canvas.draw(
                    &self.hit_circle_image,
                    DrawParam::default()
                        .dest([circle.x, circle.y])
                        .scale([scale, scale])
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

        let instructions = Text::new("Left Click: Hit | Right Click: Quit");
        canvas.draw(&instructions, DrawParam::default().dest([10.0, SCREEN_HEIGHT - 30.0]));

        canvas.finish(ctx)
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) -> GameResult {
        if button == MouseButton::Left {
            self.handle_click(x, y);
        } else if button == MouseButton::Right {
            ctx.request_quit();
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let (mut ctx, event_loop) = ContextBuilder::new("osu", "author")
        .window_setup(ggez::conf::WindowSetup::default().title("osu! Rhythm Game"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()?;

    let game = OsuGame::new(&mut ctx)?;
    event::run(ctx, event_loop, game)
}

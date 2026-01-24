# PolyRhythm - Basic Rhythm Games

This project contains 5 basic rhythm games implemented in Rust, each as a separate program:

## Games

### 1. Stepmania (`stepmania/`)
- **Controls**: D/F/J/K keys (D=Left, F=Down, J=Up, K=Right)
- **Gameplay**: Hit falling arrows as they reach the hit line
- **Features**: 4-lane gameplay, timing-based scoring, combo system

### 2. osu! (`osu/`)
- **Controls**: Mouse (Left click to hit, Right click to quit)
- **Gameplay**: Click on circles as they shrink with approach circles
- **Features**: Circle clicking, approach circles, timing-based scoring

### 3. Taiko (`taiko/`)
- **Controls**: D/F keys for Don (red), J/K keys for Kat (blue)
- **Gameplay**: Hit drum notes as they scroll from right to left
- **Features**: Drum simulation, red/blue notes, big notes

### 4. Rhythm Typer (`rhythm_typer/`)
- **Controls**: Full keyboard typing (DFJK are main rhythm keys)
- **Gameplay**: Type falling words before they pass the hit line
- **Features**: Word-based gameplay, visual typing feedback

### 5. Clone Hero (`clone_hero/`)
- **Controls**: D/F/J/K/L keys for 5 colored frets
- **Gameplay**: Hit guitar notes as they fall down the fretboard
- **Features**: 5-lane guitar gameplay, single and hold notes

## Running the Games

Each game can be run independently:

```bash
cd stepmania && cargo run
cd osu && cargo run
cd taiko && cargo run
cd rhythm_typer && cargo run
cd clone_hero && cargo run
```

## Technical Details

- **Framework**: ggez 0.9 for graphics and input handling
- **Language**: Rust
- **Dependencies**: ggez, rand, serde, serde_json
- **Graphics**: 2D rendering with meshes and text
- **Input**: Keyboard and mouse input handling

## Game Features

All games include:
- Real-time gameplay with delta time updates
- Score tracking based on timing accuracy
- Combo systems
- Visual feedback for hits/misses
- Randomly generated notes for immediate playtesting

## Status

✅ All 5 games compile successfully and are ready for testing
✅ Basic gameplay mechanics implemented
✅ DFJK control scheme implemented across applicable games
✅ Input handling working for all control schemes
✅ Visual feedback and scoring systems in place
✅ osu!Default Skin assets integrated for visual enhancement
✅ osu! timing issues fixed (circles now properly shrink over time)

## Next Steps

These basic implementations serve as foundations for:
- .osu file parsing integration
- Audio synchronization
- More complex note patterns
- Visual effects and improvements
- Game mode switching for the final PolyRhythm game

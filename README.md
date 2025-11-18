# Teleprompter - macOS Native Desktop App

A native macOS teleprompter application built with Rust, featuring window transparency, fullscreen mode, and speed controls.

## Features

- **Smooth Scrolling**: Automatic text scrolling with adjustable speed
- **Window Transparency**: Adjustable transparency for overlay recording (works in fullscreen!)
- **Fullscreen Mode**: Borderless fullscreen that maintains transparency support
- **Pause/Resume**: Control scrolling with spacebar
- **Speed Controls**: Adjust scrolling speed on the fly
- **Native Performance**: Built with Rust and Metal for optimal macOS performance

## Controls

### GUI Controls
A control panel appears on the left side with sliders for:
- **Scroll Speed**: Adjust from 10-200 px/s
- **Background Opacity**: Control transparency level
- **Pause/Resume Button**: Control scrolling
- **Reset Button**: Return to beginning

Press `H` to hide/show the control panel.

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `SPACE` | Pause/Resume scrolling |
| `↑` (Up Arrow) | Increase speed |
| `↓` (Down Arrow) | Decrease speed |
| `F` | Toggle fullscreen |
| `T` | Toggle transparency |
| `R` | Reset to beginning |
| `H` | Hide/Show control panel |
| `ESC` | Exit application |

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

## Customizing Your Script

Edit `sample_text.txt` to add your own teleprompter script.

## Requirements

- macOS 10.15 or later
- Rust 1.70 or later

## Technical Notes

- The app uses a custom borderless fullscreen mode instead of native macOS fullscreen to maintain transparency support
- Some deprecated winit 0.30 APIs are used but functionality is not affected

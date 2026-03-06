# theme-engine

`theme-engine` resolves highlight capture names to concrete styles (`fg`/`bg` RGB + bold/italic/underline) with normalization and fallback.

## Features

- Capture key normalization:
  - `@comment` and `comment` resolve the same key.
- Hierarchical fallback:
  - `comment.documentation -> comment -> normal`
- UI-role resolution:
  - supports optional `ui` map (`default_fg`, `default_bg`, `statusline`, `tab_active`, etc.)
  - falls back to legacy `styles` keys for compatibility
  - exposes `default_terminal_colors()` for terminal default fg/bg integration
- Built-in themes:
  - `tokyonight-dark`
  - `tokyonight-moon`
  - `tokyonight-light`
  - `tokyonight-day`
  - `solarized-dark`
  - `solarized-light`
- Theme loading from JSON and TOML strings.

## Quick Example

```rust
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyonight-dark")?;
    let style = theme.resolve("@comment.documentation");
    println!("style found: {}", style.is_some());
    let statusline = theme.resolve_ui("statusline");
    let active_tab = theme.resolve_ui("tab_active");
    println!(
        "ui roles: statusline={}, tab_active={}",
        statusline.is_some(),
        active_tab.is_some()
    );
    let (default_fg, default_bg) = theme.default_terminal_colors();
    println!(
        "terminal defaults available: fg={}, bg={}",
        default_fg.is_some(),
        default_bg.is_some()
    );
    Ok(())
}
```

`default_terminal_colors()` is designed to pair with terminal default-color escape sequences (OSC 10/11), for hosts that want theme-level foreground/background outside syntax spans.

## Custom Theme Example

```rust
use theme_engine::Theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = Theme::from_json_str(r#"
    {
      "styles": {
        "normal": { "fg": { "r": 220, "g": 220, "b": 220 } },
        "number": { "fg": { "r": 255, "g": 180, "b": 120 } }
      }
    }
    "#)?;

    let style = theme.resolve("number");
    println!("style found: {}", style.is_some());
    Ok(())
}
```

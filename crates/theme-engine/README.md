# theme-engine

`theme-engine` resolves highlight capture names to concrete styles (`fg`/`bg` RGB + bold/italic/underline) with normalization and fallback.

## Features

- Capture key normalization:
  - `@comment` and `comment` resolve the same key.
- Hierarchical fallback:
  - `comment.documentation -> comment -> normal`
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
    let theme = load_theme("tokyo-night")?;
    let style = theme.resolve("@comment.documentation");
    println!("style found: {}", style.is_some());
    Ok(())
}
```

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

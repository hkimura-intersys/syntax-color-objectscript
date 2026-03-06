# Usage Example Catalog

## Example UX-001: Generate Highlight Spans from ObjectScript Source

**Audience:** Developers using `highlight-spans` directly.
**Topic:** How to extract semantic spans and capture names.
**Goal:** Convert ObjectScript bytes into `HighlightResult` for downstream rendering.

### Context

- When to use it: You need parser output without choosing a render format yet.
- When not to use it: You only need final ANSI strings; use `render-ansi` orchestration helpers instead.

### Prerequisites

- Dependencies: `highlight-spans` crate.
- Config: None.
- Permissions: None.

### Example (Minimal)

**Introduction:** This example shows how to parse source and inspect captured spans.

```rust
use highlight_spans::{Grammar, SpanHighlighter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let source = b"set x = 42";
    let result = highlighter.highlight(source, Grammar::ObjectScript)?;

    for span in &result.spans {
        let capture = &result.attrs[span.attr_id].capture_name;
        println!("{} {}..{}", capture, span.start_byte, span.end_byte);
    }

    Ok(())
}
```

**Explanation:**
1. `SpanHighlighter::new` initializes the ObjectScript highlight configuration.
2. `highlight` returns `HighlightResult` with attr table and byte ranges.
3. `attr_id` indexes into `attrs` to recover capture names.

### Example (Advanced)

**Introduction:** This example highlights line-based input while preserving the same result type.

```rust
use highlight_spans::{Grammar, SpanHighlighter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut highlighter = SpanHighlighter::new()?;
    let lines = vec!["set x = 1", "set y = 2"];
    let result = highlighter.highlight_lines(&lines, Grammar::ObjectScript)?;

    assert!(!result.spans.is_empty());
    Ok(())
}
```

**Explanation:**
1. `highlight_lines` joins line slices and delegates to `highlight`.
2. The returned structure is still `HighlightResult`.
3. You can reuse the same downstream renderer pipeline.

### Evidence

- `crates/highlight-spans/src/lib.rs:133` (constructor)
- `crates/highlight-spans/src/lib.rs:228` (highlight API)
- `crates/highlight-spans/src/lib.rs:309` (line API)
- `crates/highlight-spans/src/lib.rs:86` (result type)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes (`crates/highlight-spans/src/lib.rs:719`)
- Mismatches or assumptions: None

## Example UX-002: Build and Resolve Themes with Fallback

**Audience:** Developers using `theme-engine`.
**Topic:** How to load/construct themes and resolve capture names safely.
**Goal:** Retrieve consistent `Style` values for parser captures.

### Context

- When to use it: You need stable style resolution across capture-name variants.
- When not to use it: You only need built-in theme names and no custom mapping.

### Prerequisites

- Dependencies: `theme-engine` crate.
- Config: Theme JSON/TOML text or built-in theme name.
- Permissions: None.

### Example (Minimal)

**Introduction:** This example parses JSON theme content and resolves a style.

```rust
use theme_engine::Theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = Theme::from_json_str(r#"
    {
      "styles": {
        "normal": { "fg": { "r": 200, "g": 200, "b": 200 } },
        "comment": { "italic": true }
      }
    }
    "#)?;

    let style = theme.resolve("@comment.documentation").unwrap();
    assert!(style.italic);
    Ok(())
}
```

**Explanation:**
1. `from_json_str` parses wrapped style documents.
2. `resolve` normalizes `@` prefix and case.
3. Dotted fallback maps `comment.documentation` to `comment`.

### Example (Advanced)

**Introduction:** This example loads a built-in theme alias and falls back to `normal` for unknown keys.

```rust
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyonight-dark")?;
    let style = theme.resolve("unknown.capture");
    assert!(style.is_some());
    Ok(())
}
```

**Explanation:**
1. `load_theme` accepts canonical names and aliases.
2. Unknown captures fall back to `normal` when present.
3. Built-in themes include `normal` (validated in tests).

### Evidence

- `crates/theme-engine/src/lib.rs:160` (JSON loader)
- `crates/theme-engine/src/lib.rs:136` (resolve behavior)
- `crates/theme-engine/src/lib.rs:144` (dotted fallback)
- `crates/theme-engine/src/lib.rs:207` (built-in loader)
- `crates/theme-engine/src/lib.rs:357` (alias test)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes (`crates/theme-engine/src/lib.rs:294`, `crates/theme-engine/src/lib.rs:357`)
- Mismatches or assumptions: Assumes `normal` exists in selected theme (`crates/theme-engine/src/lib.rs:150`)

## Example UX-003: End-to-End ANSI Rendering Pipeline

**Audience:** Developers integrating highlighting directly into terminal output.
**Topic:** How to run parse + theme + render in one call.
**Goal:** Produce ANSI-escaped strings from source input.

### Context

- When to use it: You want a single API that orchestrates all stages.
- When not to use it: You need custom rendering behavior beyond ANSI output.

### Prerequisites

- Dependencies: `render-ansi`, `theme-engine`.
- Config: Selected `Grammar` and a `Theme`.
- Permissions: None.

### Example (Minimal)

**Introduction:** This example renders a full source buffer to ANSI.

```rust
use highlight_spans::Grammar;
use render_ansi::highlight_to_ansi;
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyonight-dark")?;
    let out = highlight_to_ansi(b"set x = 42", Grammar::ObjectScript, &theme)?;
    println!("{}", out);
    Ok(())
}
```

**Explanation:**
1. `highlight_to_ansi` creates a highlighter and runs the full pipeline.
2. Spans are converted to `StyledSpan` entries with resolved style.
3. Renderer emits SGR sequences plus reset markers.

### Example (Advanced)

**Introduction:** This example renders per-line output for incremental terminal redraws.

```rust
use highlight_spans::Grammar;
use render_ansi::highlight_lines_to_ansi_lines;
use theme_engine::load_theme;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyonight-dark")?;
    let lines = vec!["set x = 1", "set y = 2"];
    let rendered = highlight_lines_to_ansi_lines(&lines, Grammar::ObjectScript, &theme)?;

    assert_eq!(rendered.len(), 2);
    Ok(())
}
```

**Explanation:**
1. Line API returns `Vec<String>` aligned to input line count.
2. Multi-line styled spans are clipped per line in renderer internals.
3. This is useful for terminal panes that redraw line-by-line.

### Evidence

- `crates/render-ansi/src/lib.rs:492` (orchestration API)
- `crates/render-ansi/src/lib.rs:556` (highlight call)
- `crates/render-ansi/src/lib.rs:557` (style resolution)
- `crates/render-ansi/src/lib.rs:568` (line API)
- `crates/render-ansi/src/lib.rs:460` (line clipping)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes (`crates/render-ansi/src/lib.rs:1438`, `crates/render-ansi/src/lib.rs:1456`)
- Mismatches or assumptions: None

## Example UX-004: Incremental VT Patches for IRIS SQL Terminal Updates

**Audience:** Developers integrating syntax-coloring into an interactive IRIS terminal workflow.
**Topic:** How to emit ANSI/VT patch updates instead of repainting the full frame.
**Goal:** Update only changed regions when SQL command text changes between terminal snapshots.

### Context

- When to use it: You maintain terminal state between edits and want low-latency redraws.
- When not to use it: You only need one-shot output (for example logs or static previews).

### Prerequisites

- Dependencies: `render-ansi`, `highlight-spans`, `theme-engine`.
- Config: Theme name; grammar (`sql` for SQL shell content); and either:
  - explicit viewport mode (`--origin-row`, `--origin-col`, `--width`, `--height`)
  - auto mode (omit `--origin-row`; bridge chooses stream-line or full-rerender fallback)
- Permissions: Ability to write VT escape sequences to terminal stdout.

### Example (Minimal)

**Introduction:** This example simulates two IRIS SQL shell snapshots and emits only the patch from old to new.

```bash
cat > /tmp/iris-old.sql <<'EOF'
SELECT Name, DOB
FROM Sample.Person
WHERE Home_State = :state
ORDER BY Name
EOF

cat > /tmp/iris-new.sql <<'EOF'
SELECT Name
FROM Sample.Person
WHERE Home_State = :state
ORDER BY Name
EOF

cargo run -p render-ansi --example vt_patch_bridge -- \
  /tmp/iris-new.sql tokyonight-dark sql \
  --prev /tmp/iris-old.sql \
  --width 120 --height 40 \
  --origin-row 4 --origin-col 7
```

**Explanation:**
1. `--prev` seeds the incremental renderer with the prior terminal snapshot.
2. `--origin-row/--origin-col` shift emitted cursor coordinates to the editable region (for example after a prompt).
3. The bridge highlights `iris-new.sql` and diffs it against previous styled cells.
4. Output is a VT patch (cursor-move + style + erase sequences), not a full-frame render.
5. Column movement uses display width (grapheme-aware), not raw byte count.

**Auto-mode variant (no origin):**
1. Omit `--origin-row` to let `vt_patch_bridge` choose automatically.
2. Single-line snapshots use `StreamLineRenderer`.
3. Multiline snapshots use full-rerender fallback plus relative clear/reposition patching.

### Example (Advanced)

**Introduction:** This example models IRIS Direct mode and SQL shell as sequential `READ` loops in one PTY. It resets incremental state at each `READ` boundary, then selects grammar from mode (`$ZU()` signal).

```rust
use std::io::{self, Write};

use highlight_spans::{Grammar, SpanHighlighter};
use render_ansi::IncrementalRenderer;
use theme_engine::load_theme;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IrisMode {
    Direct,
    SqlShell,
}

fn grammar_for_mode(mode: IrisMode) -> Grammar {
    match mode {
        IrisMode::Direct => Grammar::ObjectScript,
        IrisMode::SqlShell => Grammar::Sql,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theme = load_theme("tokyonight-dark")?;
    let mut highlighter = SpanHighlighter::new()?;
    let mut renderer = IncrementalRenderer::new(120, 40);
    let mut active_read_id: Option<u64> = None;

    // Replace with host events. `mode` is sampled from `$ZU()` when each READ starts.
    let events = vec![
        (100_u64, IrisMode::Direct, "set x = 2\n"),
        (100_u64, IrisMode::Direct, "set x = 23\n"),
        (101_u64, IrisMode::SqlShell, "SELECT Name, DOB\nFROM Sample.Person\n"),
        (101_u64, IrisMode::SqlShell, "SELECT Name\nFROM Sample.Person\n"),
    ];

    for (read_id, mode, snapshot) in events {
        if active_read_id != Some(read_id) {
            active_read_id = Some(read_id);
            renderer.clear_state(); // previous READ completed; start fresh
            renderer.set_origin(4, 7);
        }

        let patch = renderer.highlight_to_patch(
            &mut highlighter,
            snapshot.as_bytes(),
            grammar_for_mode(mode),
            &theme,
        )?;
        if !patch.is_empty() {
            // Write to this IRIS terminal PTY.
            print!("{patch}");
            io::stdout().flush()?;
        }
    }

    Ok(())
}
```

**Explanation:**
1. One `IncrementalRenderer` tracks the active IRIS `READ` in a single PTY.
2. Each new `read_id` marks a `READ` boundary, so `clear_state()` avoids diffing unrelated prompts.
3. `mode` (from `$ZU()`) maps to grammar: Direct -> `ObjectScript`, SQL shell -> `Sql`.
4. `highlight_to_patch` diffs only within the current `READ` lifecycle.
5. Empty patch means no terminal write is needed.

### Evidence

- `crates/render-ansi/src/lib.rs:137` (`IncrementalRenderer` state)
- `crates/render-ansi/src/lib.rs:173` (`clear_state`)
- `crates/render-ansi/src/lib.rs:181` (`set_origin`)
- `crates/render-ansi/src/lib.rs:249` (`highlight_to_patch`)
- `crates/render-ansi/src/lib.rs:1041` (display-width diff-to-patch implementation)
- `crates/render-ansi/examples/vt_patch_bridge.rs:116` (CLI option parsing including `--prev`, origin, color mode)
- `crates/render-ansi/examples/vt_patch_bridge.rs:299` (origin-row incremental branch)
- `crates/render-ansi/examples/vt_patch_bridge.rs:317` (auto-mode multiline fallback)

### Validation Notes

- Verified signature against code: Yes
- Verified usage in tests or examples: Yes (`crates/render-ansi/examples/vt_patch_bridge.rs:230`, `crates/render-ansi/src/lib.rs:1838`)
- Mismatches or assumptions: Assumes your IRIS host emits reliable `READ` boundary and mode (`$ZU()`) signals plus a clean text snapshot per update cycle.

## Doc/Code Mismatches

- No mismatches found in the documented examples versus current source/test definitions.

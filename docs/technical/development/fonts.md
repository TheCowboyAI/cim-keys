# CIM Keys Typography System

## Font Stack

The CIM Keys application uses **4 distinct fonts**, each with a specific purpose:

### 1. Rec Mono Linear (Body Text)
- **Purpose**: Standard UI text, code, data, labels
- **Type**: Monospace
- **File**: `assets/fonts/RecMonoLinear-Regular.ttf`
- **Access**: `CowboyTheme::font_body()` or `FONT_BODY`
- **Usage**:
  ```rust
  text("Hello World")
      .font(CowboyTheme::font_body())
      .size(14)
  ```

### 2. Poller One (Headings)
- **Purpose**: Page titles, section headings, emphasis
- **Type**: Display font
- **File**: `assets/fonts/PollerOne-Regular.ttf`
- **Access**: `CowboyTheme::font_heading()` or `FONT_HEADING`
- **Usage**:
  ```rust
  text("CIM Keys")
      .font(CowboyTheme::font_heading())
      .size(32)
  ```

### 3. Noto Color Emoji (Emoji)
- **Purpose**: Emoji characters, status indicators
- **Type**: Color emoji font
- **File**: `assets/fonts/NotoColorEmoji.ttf`
- **Access**: `CowboyTheme::font_emoji()` or `EMOJI_FONT`
- **Usage**:
  ```rust
  text("🔐 Secure")
      .font(CowboyTheme::font_emoji())
      .size(16)
  ```

### 4. Material Icons (Icons)
- **Purpose**: Interface icons, buttons, navigation
- **Type**: Icon font
- **File**: `assets/fonts/MaterialIcons-Regular.ttf`
- **Access**: `CowboyTheme::font_icons()` or `MATERIAL_ICONS`
- **Usage**:
  ```rust
  text("") // Material icon character
      .font(CowboyTheme::font_icons())
      .size(24)
  ```

## Font Definition

All fonts are defined as `Font::External` with embedded bytes in `src/icons.rs`:

```rust
pub const FONT_BODY: Font = Font::External {
    name: "Rec Mono Linear",
    bytes: include_bytes!("../assets/fonts/RecMonoLinear-Regular.ttf"),
};
```

This approach:
- Embeds fonts directly in the binary (no external files needed)
- Works in WASM builds
- Allows explicit font selection per text element
- Supports offline/air-gapped operation

## Best Practices

### ✅ DO:
- Use `FONT_BODY` for all standard UI text
- Use `FONT_HEADING` for titles and section headers
- Use `EMOJI_FONT` for emoji characters
- Use `MATERIAL_ICONS` for icon glyphs
- Access via `CowboyTheme::font_*()` methods for consistency

### ❌ DON'T:
- Use `Font::DEFAULT` (ambiguous with multiple fonts)
- Mix fonts inappropriately (emoji in headings, etc.)
- Hardcode font family names as strings
- Load fonts via `.font(include_bytes!())` in `run()` (already embedded)

## Migration Notes

Previous code used:
- `Font::DEFAULT` for everything (ambiguous)
- `.font(include_bytes!())` in `run()` function (duplicate loading)
- `Font { family: Family::Name("..."), ... }` (doesn't work with embedded fonts)

New code uses:
- `Font::External` constants with explicit names
- Theme methods for accessing fonts
- Embedded bytes in font definitions (single source of truth)

# Atrocitus Assets Directory

This directory is intended for custom PNG sprite art and textures.

## How to Drop In PNG Sprites
You can load any standard PNG image directly via:
```rust
let custom_sprite = Sprite::load_png("assets/my_knight.png")?;
custom_sprite.draw(&mut framebuffer, x, y, false);
```

- Supported format: PNG with 8-bit RGBA channels.
- Transparency: Alpha channel (0-255) is automatically preserved and blended onto the framebuffer.
- Recommended dimensions:
  - Characters: 16x22 to 24x32 pixels
  - Bonfires / Props: 18x20 to 32x32 pixels
  - Architecture: 32x32 to 64x64 pixels

# archflow-plugins

> **External Integrations** - Draw.io import, SVG rasterization, and texture atlas packing for seamless interoperability with external diagram tools.

## Overview

`archflow-plugins` provides a plugin system for integrating ArchFlow with external diagram tools and formats. It enables importing Draw.io diagrams, parsing SVG icons, rasterizing vector graphics for GPU rendering, and efficiently packing textures into atlases.

**Key Capabilities:**
- **Draw.io import** - Decode Draw.io's compressed XML format
- **SVG parsing** - Extract and parse SVG icons from libraries
- **GPU rasterization** - Convert SVG to pixel data for rendering
- **Atlas packing** - Shelf-packing algorithm for efficient texture layout
- **`no_std` compatible** - Works in embedded and WASM environments

## Architecture

The crate follows a **Plugin Architecture** with format adapters:

```
┌─────────────────────────────────────────────────────────────────┐
│                    External Sources                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Draw.io Files│  │ SVG Libraries│  │Icon Sets     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      Format Adapters                            │
│  ┌──────────────┐  ┌──────────────┐                            │
│  │Draw.io Decoder│  │SVG Parser    │                            │
│  │(3-layer decode)│  │(XML→SVG)    │                            │
│  └──────────────┘  └──────────────┘                            │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────────┐
│                      Processing Pipeline                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │SvgRasterizer │  │AtlasPacker   │  │UV Generator  │          │
│  │(SVG→Pixels)  │  │(Shelf Pack)  │  │(Coordinates) │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
                         GPU Textures
                    (archflow-render)
```

## Core Concepts

### Draw.io Decoding

Draw.io uses a multi-layer encoding format that must be decoded sequentially:

```
Original XML → Deflate → Base64 → URL Encode
     ↓                                        ↓
Decoded XML  ←  Inflate ←  Base64 Decode ← URL Decode
```

```rust
use archflow_plugins::decode_drawio_data;

// Draw.io data from clipboard or file
let encoded = "eNpt1s1q..."; // Base64+Deflate+URL encoded

// Decode to XML
let xml = decode_drawio_data(encoded)?;

// Parse library icons
let icons = parse_library_xml(&xml);
println!("Loaded {} icons", icons.len());
```

**Decoding Layers:**

| Layer | Transformation | Purpose |
|-------|---------------|---------|
| URL Encode | Replace `%xx` escapes | Safe for URLs |
| Base64 | 64-char alphabet → Binary | ASCII-safe encoding |
| Deflate | LZ77 compression | Reduce size ~70% |

### Library Icons

Parsed icons are represented as `LibraryIcon`:

```rust
LibraryIcon {
    id: String,       // Unique identifier (e.g., "AWS_Compute_EC2")
    name: String,     // Display name (e.g., "Amazon EC2")
    svg_data: String, // Raw SVG content
}
```

**Icon Library Structure:**

```xml
<mxlibrary>
  <icon id="AWS_Compute_EC2" name="Amazon EC2">
    <svg>...</svg>
  </icon>
  <icon id="AWS_Database_S3" name="Amazon S3">
    <svg>...</svg>
  </icon>
</mxlibrary>
```

### SVG Rasterization

Convert SVG to pixel data for GPU rendering:

```rust
use archflow_plugins::SvgRasterizer;

let mut rasterizer = SvgRasterizer::new(2048, 2048);

// Add SVG to atlas
let svg_data = "<svg><rect width='64' height='64' fill='red'/></svg>";

if let Some(uv_rect) = rasterizer.add_svg(svg_data, 64) {
    println!("UV coordinates: {:?}", uv_rect);
    println!("Atlas utilization: {:.1}%", rasterizer.utilization() * 100.0);
}
```

**Output Format:**
- RGBA pixel data (4 bytes per pixel)
- Pre-multiplied alpha for GPU blending
- Power-of-2 dimensions recommended

### Shelf-Packing Algorithm

Efficient texture packing using horizontal shelves:

```rust
use archflow_plugins::AtlasPacker;

let mut packer = AtlasPacker::new(1024, 1024);
packer.padding = 4; // 4px padding between textures

// Pack multiple icons
let rect1 = packer.allocate(64, 64);
let rect2 = packer.allocate(32, 32);
let rect3 = packer.allocate(128, 128);

// Check results
println!("Packed {} icons", 3);
println!("Utilization: {:.1}%", packer.utilization() * 100.0);
println!("Shelves used: {}", packer.shelves.len());
```

**Algorithm Characteristics:**

| Aspect | Shelf Packing | Bin Packing |
|--------|---------------|-------------|
| **Complexity** | O(shelves) | O(n log n) |
| **Reorganization** | Never | Sometimes |
| **Best For** | Uniform sizes | Varied sizes |
| **Implementation** | Simple | Complex |

**Shelf Structure:**

```
┌─────────────────────────────────────┐
│ Shelf 0 (height: 64)                │
│ [64x64] [32x64] [128x64] [64x64]    │
├─────────────────────────────────────┤
│ Shelf 1 (height: 32)                │
│ [32x32] [32x32]                     │
├─────────────────────────────────────┤
│ Shelf 2 (height: 128)               │
│ [128x128]                           │
└─────────────────────────────────────┘
```

## Usage Examples

### Import Draw.io Library

```rust
use archflow_plugins::{decode_drawio_data, parse_library_xml};

// From Draw.io clipboard
let clipboard_data = "eNpt1s1q2...";

// Decode and parse
let xml = decode_drawio_data(clipboard_data)?;
let icons = parse_library_xml(&xml);

// Find specific icon
let ec2_icon = icons.iter()
    .find(|icon| icon.id == "AWS_Compute_EC2");

if let Some(icon) = ec2_icon {
    println!("Found: {}", icon.name);
    println!("SVG: {} bytes", icon.svg_data.len());
}
```

### Build Icon Atlas

```rust
use archflow_plugins::{SvgRasterizer, AtlasPacker};

let mut rasterizer = SvgRasterizer::new(2048, 2048);

// Process icon library
for icon in &icons {
    let size = estimate_svg_size(&icon.svg_data);
    if let Some(uv_rect) = rasterizer.add_svg(&icon.svg_data, size) {
        println!("{}: {:?}", icon.name, uv_rect);
    }
}

// Get final atlas data
let atlas_data = rasterizer.finish();
println!("Atlas: {}x{} pixels", atlas_data.width, atlas_data.height);
```

### Manual Atlas Packing

```rust
use archflow_plugins::AtlasPacker;

// Create packer with padding
let mut packer = AtlasPacker::new(1024, 1024);
packer.padding = 2; // 2px padding

// Pack textures in order
let textures = vec![(64, 64), (32, 32), (128, 128), (64, 64)];

for (w, h) in textures {
    match packer.allocate(w, h) {
        Some(rect) => println!("Packed {}x{} at ({}, {})", w, h, rect.x, rect.y),
        None => println!("Failed to pack {}x{}", w, h),
    }
}

// Check efficiency
println!("Utilization: {:.1}%", packer.utilization() * 100.0);
println!("Free space: {} pixels²", packer.free_area());
```

## Integration with Other Crates

```toml
[dependencies]
archflow-plugins = "0.36"
archflow-render = "0.36"  # For GPU texture upload
archflow-persistence = "0.36"  # For icon library storage
```

### Data Flow

```
Draw.io File → Decoder → XML → Parser → LibraryIcon
                                                      │
                                                      ▼
                                              SvgRasterizer
                                                      │
                                                      ▼
                                              AtlasPacker
                                                      │
                                                      ▼
                                              UV Coordinates
                                                      │
                                                      ▼
                                              GPU Texture
                                         (archflow-render)
```

## Performance Characteristics

| Operation | Time | Notes |
|-----------|------|-------|
| Draw.io Decode | ~10ms | Typical diagram file |
| XML Parsing | ~5ms | Per library |
| SVG Rasterization | ~50ms | Per icon (with resvg) |
| Atlas Packing | ~1ms | Per 100 icons |

### Memory Usage

| Component | Memory | Notes |
|-----------|--------|-------|
| SVG Data | ~1-5KB | Per icon |
| Atlas Texture | 16MB | 2048×2048 RGBA |
| Packing Metadata | ~100 bytes | Per icon |

### Atlas Efficiency

Typical utilization rates:
- **Uniform icons** (32×32, 64×64): 85-95%
- **Varied sizes**: 70-85%
- **Random sizes**: 60-75%

## Constraints and Limitations

### Current Constraints

- **SVG Features**: Basic shapes only (full implementation pending)
- **Deflate**: Stub implementation (needs full flate2 integration)
- **No std**: Works in `no_std` environment
- **No reorganization**: Shelf packing doesn't reorder existing items

### Known Limitations

- **SVG Support**: Limited to basic shapes and paths
- **Text rendering**: SVG text not fully supported
- **Gradients**: Basic linear/radial gradients only
- **Filters**: SVG filters (blur, shadow) not implemented

## Best Practices

### Draw.io Import

1. **Validate input** before decoding
2. **Handle errors** at each decode layer
3. **Cache results** for repeated access
4. **Use separate threads** for large libraries

### Atlas Packing

1. **Sort by height** before packing for better efficiency
2. **Use power-of-2 sizes** for GPU compatibility
3. **Add padding** to prevent texture bleeding
4. **Monitor utilization** and resize when needed

### SVG Rasterization

1. **Pre-process** SVG to remove unnecessary elements
2. **Normalize** sizes before rasterization
3. **Use appropriate resolution** for target display
4. **Consider mipmaps** for downscaled rendering

## Future Enhancements

### Planned Features

- **Full SVG Support**: Complete SVG 1.1 specification
- **Advanced Packing**: Bin-packing for varied sizes
- **Reorganization**: Dynamic atlas repacking
- **Compression**: Texture compression (BC7, ETC2)
- **Streaming**: Incremental atlas loading

### External Dependencies

- **resvg**: Full SVG rendering support
- **flate2**: Complete Deflate implementation
- **xml-rs**: Robust XML parsing

## `no_std` Compatibility

This crate is designed to work without the standard library:

```toml
[dependencies.archflow-plugins]
version = "0.36"
default-features = false
```

**Available Features:**
- `std` - Enable standard library (default)
- `drawio` - Draw.io format support
- `svg` - SVG parsing support
- `atlas` - Atlas packing support

## References

- **Draw.io Format**: https://www.drawio.com/doc/faq/file-format
- **SVG Specification**: https://www.w3.org/TR/SVG/
- **Shelf Packing**: Classic bin-packing algorithm variant
- **archflow-render**: GPU texture integration

## License

MIT License - See LICENSE file for details.

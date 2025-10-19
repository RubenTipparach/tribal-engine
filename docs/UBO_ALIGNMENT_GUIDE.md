# UBO Alignment Guide

## Overview
This document explains the std140 layout rules for Uniform Buffer Objects (UBOs) in GLSL and how to ensure proper alignment between Rust structs and shader uniforms.

## The Problem
Vulkan shaders use **std140 layout** for uniform buffers, which has strict alignment rules. If the Rust-side struct doesn't match these rules exactly, the GPU will read incorrect data from memory, leading to visual corruption or incorrect values.

### Symptoms of Misalignment
- Colors appearing incorrect (e.g., green tint when expecting purple/blue)
- Random or garbage values in shader uniforms
- Seemingly correct values in Rust debug output, but wrong results on GPU

## std140 Layout Rules

### Basic Type Sizes
| Type | Size | Alignment |
|------|------|-----------|
| `float` | 4 bytes | 4 bytes |
| `vec2` | 8 bytes | 8 bytes |
| `vec3` | **12 bytes** | **16 bytes** ⚠️ |
| `vec4` | 16 bytes | 16 bytes |
| `mat4` | 64 bytes | 16 bytes |

### Key Rules
1. **vec3 occupies 16 bytes** (12 bytes of data + 4 bytes padding)
2. **vec3 must start at a 16-byte aligned offset**
3. A scalar (float) following a vec3 fits in the vec3's padding slot (no extra padding needed)
4. Multiple consecutive scalars require explicit padding before the next vec3

## Example: Skybox UBO

### Correct Implementation

**Rust struct** (`src/background.rs`):
```rust
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SkyboxUniformBufferObject {
    pub model: Mat4,                    // offset 0, size 64
    pub view: Mat4,                     // offset 64, size 64
    pub proj: Mat4,                     // offset 128, size 64
    pub view_pos: Vec3,                 // offset 192, size 12
    pub star_density: f32,              // offset 204, size 4 (fills vec3 padding)
    pub star_brightness: f32,           // offset 208, size 4
    pub _pad0: [f32; 3],                // offset 212, size 12 ⚠️ CRITICAL!
    pub nebula_primary_color: Vec3,     // offset 224, size 12 (16-byte aligned ✓)
    pub nebula_intensity: f32,          // offset 236, size 4 (fills vec3 padding)
    pub nebula_secondary_color: Vec3,   // offset 240, size 12 (16-byte aligned ✓)
    pub background_brightness: f32,     // offset 252, size 4 (fills vec3 padding)
}
// Total size: 256 bytes

unsafe impl bytemuck::Pod for SkyboxUniformBufferObject {}
unsafe impl bytemuck::Zeroable for SkyboxUniformBufferObject {}
```

**GLSL shader** (`shaders/skybox_starry.frag`):
```glsl
layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec3 viewPos;
    float starDensity;              // follows vec3, fits in padding slot
    float starBrightness;
    float pad0;                     // explicit padding
    float pad1;                     // explicit padding
    float pad2;                     // need 3 floats (12 bytes) to reach next 16-byte boundary
    vec3 nebulaPrimaryColor;        // now at 16-byte aligned offset ✓
    float nebulaIntensity;          // follows vec3, fits in padding slot
    vec3 nebulaSecondaryColor;      // 16-byte aligned ✓
    float backgroundBrightness;     // follows vec3, fits in padding slot
} ubo;
```

### Why 3 Padding Floats?

After `star_brightness` at offset 208:
- Next byte is offset 212
- Next 16-byte boundary is 224
- Need: 224 - 212 = **12 bytes** of padding
- Solution: `[f32; 3]` or 3 `float` fields in GLSL

## Step-by-Step Alignment Process

### 1. Calculate Offsets
Track the memory offset as you add each field:

```
model:          offset 0   → 64 bytes  → next offset: 64
view:           offset 64  → 64 bytes  → next offset: 128
proj:           offset 128 → 64 bytes  → next offset: 192
view_pos:       offset 192 → 12 bytes  → next offset: 204
star_density:   offset 204 → 4 bytes   → next offset: 208
star_brightness:offset 208 → 4 bytes   → next offset: 212
```

### 2. Check Alignment Before vec3
Before adding `nebula_primary_color` (a vec3):
- Current offset: 212
- vec3 requires 16-byte alignment
- Next 16-byte boundary: 224 (since 212 % 16 = 12)
- Padding needed: 224 - 212 = **12 bytes**

### 3. Add Padding
```rust
pub _pad0: [f32; 3],  // 12 bytes padding
```

### 4. Verify Alignment
```
_pad0:                 offset 212 → 12 bytes → next offset: 224
nebula_primary_color:  offset 224 ✓ (224 % 16 = 0, properly aligned!)
```

## Pattern: Scalar After vec3

When a scalar follows a vec3, it fits in the vec3's padding slot:
```rust
pub some_color: Vec3,      // offset X, takes 16 bytes (12 data + 4 padding)
pub some_value: f32,       // offset X+12, uses the padding slot
```

GLSL equivalent:
```glsl
vec3 someColor;     // takes 16 bytes
float someValue;    // fits in the 4th slot, no extra padding needed
```

## Checklist for New UBOs

When creating a new UBO:

1. **Write the Rust struct first** with `#[repr(C)]`
2. **Calculate offsets manually** or use `offset_of!` macro
3. **Check alignment before every vec3**:
   - Current offset % 16 == 0? ✓ Good
   - Current offset % 16 != 0? ⚠️ Add padding
4. **Add padding fields** as `[f32; N]` where N = (next_16_byte_boundary - current_offset) / 4
5. **Match GLSL layout exactly** with same number of padding floats
6. **Verify with test program** (see below)
7. **Add `bytemuck::Pod` and `bytemuck::Zeroable`** traits

## Testing UBO Layout

Create a test program to verify offsets:

```rust
use std::mem::{offset_of, size_of};

#[repr(C)]
struct TestUBO {
    // ... your struct fields
}

fn main() {
    println!("Total size: {}", size_of::<TestUBO>());
    println!("nebula_primary offset: {}", offset_of!(TestUBO, nebula_primary_color));

    // Check: offset % 16 should be 0 for all vec3 fields
    assert_eq!(offset_of!(TestUBO, nebula_primary_color) % 16, 0);
}
```

## Common Mistakes

### ❌ Wrong: Insufficient Padding
```rust
pub star_brightness: f32,
pub _pad0: [f32; 2],        // Only 8 bytes - NOT ENOUGH!
pub nebula_primary: Vec3,   // Will be at offset 220 (not 16-byte aligned)
```

### ❌ Wrong: GLSL and Rust Mismatch
```rust
// Rust
pub _pad0: [f32; 3],  // 3 floats

// GLSL
float pad0;           // Only 1 float - MISMATCH!
float pad1;
```

### ✅ Correct: Matching Padding
```rust
// Rust
pub _pad0: [f32; 3],  // 3 floats = 12 bytes

// GLSL
float pad0;           // 3 floats total = 12 bytes
float pad1;
float pad2;
```

## Quick Reference

### Formula for Padding Before vec3
```
current_offset = last_field_offset + last_field_size
next_aligned = ((current_offset + 15) / 16) * 16  // Round up to next 16
padding_bytes = next_aligned - current_offset
padding_floats = padding_bytes / 4
```

### Memory Layout Visualization
```
Offset  | Field                | Size | Alignment
--------|---------------------|------|----------
0       | mat4 model          | 64   | ✓
64      | mat4 view           | 64   | ✓
128     | mat4 proj           | 64   | ✓
192     | vec3 view_pos       | 12   | ✓ (192 % 16 = 0)
204     | float star_density  | 4    | (fills vec3 padding)
208     | float star_bright   | 4    |
212     | [PADDING: 12 bytes] |      | (to reach 224)
224     | vec3 nebula_primary | 12   | ✓ (224 % 16 = 0)
236     | float nebula_int    | 4    | (fills vec3 padding)
240     | vec3 nebula_sec     | 12   | ✓ (240 % 16 = 0)
252     | float bg_bright     | 4    | (fills vec3 padding)
```

## Debugging Misalignment

If you see incorrect values on the GPU:

1. **Add debug output** in Rust to print actual UBO values
2. **Test with hardcoded values** in the shader to isolate the issue
3. **Calculate offsets** using `offset_of!` macro
4. **Verify alignment** of all vec3 fields (offset % 16 == 0)
5. **Check struct size** matches expected (should be multiple of 16)
6. **Compare Rust and GLSL** field-by-field

## Additional Resources

- [Vulkan std140 Layout Rules](https://www.khronos.org/registry/OpenGL/specs/gl/glspec45.core.pdf) (Section 7.6.2.2)
- [Rust bytemuck crate](https://docs.rs/bytemuck/latest/bytemuck/)
- [GLSL std140 Calculator](https://www.ibiblio.org/e-notes/webgl/gpu/uniform.htm)

## Version History

- **2025-10-19**: Initial documentation after fixing skybox green tint bug
  - Issue: `_pad0: [f32; 2]` (8 bytes) caused misalignment
  - Fix: Changed to `_pad0: [f32; 3]` (12 bytes) for proper vec3 alignment

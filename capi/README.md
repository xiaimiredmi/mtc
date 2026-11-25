# Using MTC from a game

`mtc.dll` plus `include/mtc.h`: decode a `.mtc` straight into a texture, or encode one
at load time. No dependencies, no global state, nothing on disk.

```c
#include "mtc.h"

bool load_texture(const uint8_t* file, int32_t size) {
    uint8_t* rgba; int32_t w, h;
    if (mtc_decode(file, size, &rgba, &w, &h) != MTC_OK) {
        log("mtc: %s", mtc_last_error());
        return false;
    }
    upload_to_gpu(rgba, w, h);   // your engine's texture upload, at native size
    mtc_free(rgba);
    return true;
}
```

There is deliberately no size argument: the picture comes back at its own size and the
engine scales it. Scaling inside the codec is a separate call
(`mtc_decode_scaled`) for mip chains, not something the caller has to think about.

## Why it is shaped like this

* **One call per image.** `mtc_decode_rgba` returns a buffer you own; `mtc_free` takes
  only the pointer, because the length is stored in front of the data. Forgetting to
  pass a size back is how C APIs usually leak.
* **No size knob.** `mtc_decode` returns the picture at its own size; the engine scales
  it. If you do want the codec to shrink while decoding - for a mip chain, where it
  builds the small picture directly instead of decoding then resizing - `mtc_decode_scaled`
  takes a maximum long edge.
* **`mtc_decode_rgb`** gives 3 bytes per pixel. For opaque textures that is a third
  less to move to the GPU.
* **`mtc_decode_region`** decodes one rectangle and skips the transform elsewhere.
* **Thread safe.** No shared state; decode different images on different worker
  threads. The error message is per thread.
* **No unwinding.** Every entry point catches panics and returns `MTC_ERR_INTERNAL`, so
  a corrupt file cannot take the process down.

## Numbers

Single core of an i5-6440HQ, release build, 4:2:0, quality 85 — decode of the whole
picture, not a thumbnail:

| picture | before | after |
| --- | --- | --- |
| 512×512 | 15 ms | 11 ms |
| 1024×1024 | 55 ms | 40 ms |
| 2048×2048 | 208 ms | 149 ms |

That is about 28 MP/s, up from 20. A phone core is roughly a third of that, so budget
about 100 ms for a 1024² texture on mobile — a loading screen, not a frame.

Where the time goes at 2048² (measured with `MTCODEC_PROFILE=1`):

| stage | ms |
| --- | --- |
| entropy decoding | 50 |
| inverse DCT + pixel store | 56 |
| colour conversion, plane allocation | 43 |

Two changes made the difference, both in `rust/src/transform.rs`:

* the inverse DCT moved from `f64` matrix multiplies to `f32` with both passes reading
  straight lines (the first pass writes its transpose);
* the pixel rounding stopped calling `f32::round`, which is a libm call unless the
  target enables SSE4.1 — it was costing more than the transform itself. Adding half a
  step with the sign and letting the cast truncate is the same rounding, a third of
  the time.

## Building

```powershell
powershell -ExecutionPolicy Bypass -File capi\build.ps1      # -> capi\build\mtc.dll + mtc.h
```

The DLL is also installed next to the viewer by `installer\MtcSetup.exe`, so a game
built on the same machine can link against it directly.

## Unity

```csharp
[DllImport("mtc", CallingConvention = CallingConvention.Cdecl)]
static extern int mtc_decode_rgba(byte[] data, int len, int maxSide,
                                  out IntPtr rgba, out int w, out int h);
[DllImport("mtc", CallingConvention = CallingConvention.Cdecl)]
static extern void mtc_free(IntPtr p);
```

Drop `mtc.dll` into `Assets/Plugins/x86_64/`, then `Texture2D.LoadRawTextureData`
from the buffer before `mtc_free`. Unreal and Godot work the same way through their
respective `LoadLibrary`/PInvoke layers.

## What it does not do yet

* No partial decode: the whole picture, or a whole picture scaled down. There is no
  "decode this rectangle" call.
* No streaming: the file has to be in memory. Fine for textures, not for a 100 MP
  panorama.
* Alpha under fully transparent pixels is zeroed by the encoder (the PSNR metric
  ignores those pixels); alpha itself is exact, and lossless mode is pixel exact for
  colour that is visible.

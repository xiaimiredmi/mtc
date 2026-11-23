/*
 * mtc.h - use the MTC image codec from C, C++, C# or a game engine.
 *
 * Link mtc.dll (or libmtc.so) and include this file. Everything is `cdecl`, there is
 * no global state, and decode/encode never touch the filesystem: hand in a buffer,
 * get a buffer back.
 *
 *   MtcInfo info;
 *   if (mtc_info(bytes, len, &info) == MTC_OK) { ... }
 *
 *   uint8_t* rgba; int32_t w, h;
 *   if (mtc_decode_rgba(bytes, len, &rgba, &w, &h) == MTC_OK) {
 *       upload_to_gpu(rgba, w, h);   // or memcpy into your texture staging buffer
 *       mtc_free(rgba);
 *   } else {
 *       log(mtc_last_error());
 *   }
 *
 * Buffers returned through `out` pointers are owned by the caller and must be
 * released with mtc_free. Each is prefixed with its own length, so mtc_free needs no
 * size argument - the usual way this gets leaked or double-counted.
 *
 * Decoding is pure computation with no locks: call it from as many worker threads as
 * you like, on different images.
 */
#ifndef MTC_H
#define MTC_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _WIN32
#  ifdef MTC_BUILD
#    define MTC_API __declspec(dllexport)
#  else
#    define MTC_API __declspec(dllimport)
#  endif
#else
#  define MTC_API __attribute__((visibility("default")))
#endif

/* Return codes. Anything but MTC_OK is a failure. */
#define MTC_OK              0
#define MTC_ERR_ARGS       -1  /* null pointer, zero size, bad quality */
#define MTC_ERR_FORMAT     -2  /* not an .mtc file, or a version we cannot read */
#define MTC_ERR_TRUNCATED  -3  /* the file ends in the middle of something */
#define MTC_ERR_TOO_LARGE  -4  /* declared size beyond what the decoder accepts */
#define MTC_ERR_ALLOC      -5  /* the host refused the allocation */
#define MTC_ERR_INTERNAL   -6  /* a bug; mtc_last_error has the detail */

/* What a file says about itself. `mode` is the container mode, 1..5. */
typedef struct MtcInfo {
    int32_t width;
    int32_t height;
    int32_t channels;   /* 1 grey, 3 colour, 4 with alpha */
    int32_t mode;
    int32_t quality;
    int32_t version;
    int32_t has_alpha;
    int32_t has_aq;
} MtcInfo;

/* Read the header only. Cheap: no coefficients are decoded. */
MTC_API int32_t mtc_info(const uint8_t* data, int32_t len, MtcInfo* out);

/*
 * Decode to RGBA8, row-major, top row first, 4 bytes per pixel.
 *
 * `max_side` > 0 decodes straight to a smaller picture whose longest side is at most
 * that; 0 decodes full size. The scaled path builds the small picture directly
 * instead of decoding and shrinking, so a thumbnail of a 12 MP photo does not cost a
 * 12 MP buffer - use it for mip chains and for level-of-detail previews.
 */
MTC_API int32_t mtc_decode_rgba(const uint8_t* data, int32_t len, int32_t max_side,
                                uint8_t** out_rgba, int32_t* out_w, int32_t* out_h);

/* Same, but 3 bytes per pixel. Alpha, if the file had any, is composited over black
 * and dropped. Use this for opaque textures: it is a third less memory to move. */
MTC_API int32_t mtc_decode_rgb(const uint8_t* data, int32_t len, int32_t max_side,
                               uint8_t** out_rgb, int32_t* out_w, int32_t* out_h);

/*
 * Decode one rectangle. Same contract as mtc_decode_rgba, but only the pixels inside
 * (x, y, w, h) are reconstructed: the coefficient stream is walked in full - it is
 * sequential - yet the transform and colour conversion for everything else is
 * skipped. A quarter of a 2048x2048 picture costs about 60% of the whole.
 */
MTC_API int32_t mtc_decode_region(const uint8_t* data, int32_t len, int32_t x, int32_t y,
                                  int32_t w, int32_t h, uint8_t** out_rgba,
                                  int32_t* out_w, int32_t* out_h);

/*
 * Encode pixels: `channels` is 1 (grey), 3 (RGB) or 4 (RGBA).
 *
 * `quality` is 1..100; 100 is not lossless - for that pass `lossless` = 1, which is
 * pixel exact and much bigger. A fully opaque RGBA image is stored as RGB
 * automatically, because an alpha channel that is 255 everywhere is only bytes.
 */
MTC_API int32_t mtc_encode(const uint8_t* pixels, int32_t w, int32_t h, int32_t channels,
                           int32_t quality, int32_t lossless, uint8_t** out, int32_t* out_len);

/* Release a buffer this library handed out. Null is a no-op. */
MTC_API void mtc_free(uint8_t* p);

/* The last error text for this thread; never null, never to be freed. */
MTC_API const char* mtc_last_error(void);

/* Library version, for a loading screen or a log line. */
MTC_API const char* mtc_version(void);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* MTC_H */

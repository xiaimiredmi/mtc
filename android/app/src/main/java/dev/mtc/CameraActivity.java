package dev.mtc;

import android.Manifest;
import android.app.Activity;
import android.content.Context;
import android.content.pm.PackageManager;
import android.graphics.ImageFormat;
import android.graphics.SurfaceTexture;
import android.hardware.camera2.CameraAccessException;
import android.hardware.camera2.CameraCaptureSession;
import android.hardware.camera2.CameraCharacteristics;
import android.hardware.camera2.CameraDevice;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.CaptureRequest;
import android.hardware.camera2.params.StreamConfigurationMap;
import android.media.Image;
import android.media.ImageReader;
import android.os.Bundle;
import android.os.Handler;
import android.os.HandlerThread;
import android.os.Looper;
import android.util.Size;
import android.view.Gravity;
import android.view.Surface;
import android.view.TextureView;
import android.view.View;
import android.view.ViewGroup;
import android.widget.FrameLayout;
import android.widget.TextView;
import android.widget.Toast;

import java.io.File;
import java.nio.ByteBuffer;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;

/**
 * The camera: Camera2 preview, one still at a time, encoded straight to `.mtc`.
 *
 * <p>The still arrives as {@code YUV_420_888}, so no JPEG is ever produced: the
 * planes are copied out and the conversion plus coding happen in Rust. A large
 * sensor frame takes a moment on a phone, so the work runs on a worker thread
 * and the header line reports the time it took.
 */
public class CameraActivity extends Activity {
    private static final int REQ_CAMERA = 1;
    /** Above this many pixels the still is captured smaller, to keep it snappy. */
    /** Upper bound on still size while 8 MP is selected; the setting can raise it. */
    private static final int MAX_PIXELS_DEFAULT = 3264 * 2448;
    private static final int[][] SIZE_CHOICES = {
            {3264 * 2448, 0},        // 8 MP, the default: cheap to decode on a phone
            {4080 * 3060, 1},        // 12 MP
            {Integer.MAX_VALUE, 2},  // whatever the sensor offers
    };

    private final Handler ui = new Handler(Looper.getMainLooper());
    private HandlerThread worker;
    private Handler cameraHandler;

    private FrameLayout root;
    private TextureView preview;
    private TextView hud;
    private TextView rangeToggle;
    private TextView sizeToggle;
    private TextView qualityToggle;
    private CameraDevice device;
    private CameraCaptureSession session;
    private ImageReader stills;
    private Size stillSize = new Size(1920, 1080);
    private Size previewSize = new Size(1280, 720);
    private String cameraId;
    private boolean busy;
    /** Encoder quality; persisted, because 88 is a guess and 95 is often worth it. */
    private int quality = 88;
    private int sizeChoice;
    private int sensorOrientation = 90;
    /** BT.601 range of the frames this device hands out. Devices disagree, so
     *  the camera screen has a switch and the choice is remembered. */
    private boolean fullRange;
    /** Which way the camera points; a front camera also mirrors its picture. */
    private boolean front;

    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);

        root = new FrameLayout(this);
        root.setBackgroundColor(0xFF000000);

        preview = new TextureView(this);
        root.addView(preview, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));

        hud = new TextView(this);
        hud.setTextColor(0xFFEDEDF0);
        hud.setTextSize(12);
        hud.setPadding(dp(16), dp(14), dp(16), dp(14));
        hud.setBackgroundColor(0x66000000);
        root.addView(hud, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT,
                Gravity.TOP));

        TextView shutter = new TextView(this);
        shutter.setText("●");
        shutter.setTextColor(0xFFFFFFFF);
        shutter.setTextSize(44);
        shutter.setGravity(Gravity.CENTER);
        shutter.setBackgroundColor(0x33FFFFFF);
        FrameLayout.LayoutParams lp = new FrameLayout.LayoutParams(dp(92), dp(92),
                Gravity.BOTTOM | Gravity.CENTER_HORIZONTAL);
        lp.setMargins(0, 0, 0, dp(28));
        root.addView(shutter, lp);
        shutter.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                capture();
            }
        });

        TextView close = new TextView(this);
        close.setText("✕");
        close.setTextColor(0xFFFFFFFF);
        close.setTextSize(22);
        close.setGravity(Gravity.CENTER);
        FrameLayout.LayoutParams blp = new FrameLayout.LayoutParams(dp(48), dp(48),
                Gravity.TOP | Gravity.END);
        blp.setMargins(0, dp(56), dp(16), 0);
        root.addView(close, blp);
        close.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                finish();
            }
        });

        setContentView(root);

        TextView swap = new TextView(this);
        swap.setText("⇄");
        swap.setTextColor(0xFFEDEDF0);
        swap.setTextSize(20);
        swap.setGravity(Gravity.CENTER);
        swap.setBackgroundColor(0x33FFFFFF);
        FrameLayout.LayoutParams slp = new FrameLayout.LayoutParams(dp(52), dp(52),
                Gravity.BOTTOM | Gravity.END);
        slp.setMargins(0, 0, dp(20), dp(46));
        root.addView(swap, slp);
        swap.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                front = !front;
                closeCamera();
                openCamera();
            }
        });

        rangeToggle = new TextView(this);
        rangeToggle.setTextColor(0xFFEDEDF0);
        rangeToggle.setTextSize(12);
        rangeToggle.setPadding(dp(16), dp(10), dp(16), dp(10));
        rangeToggle.setBackgroundColor(0x66000000);
        FrameLayout.LayoutParams rlp = new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT,
                Gravity.BOTTOM | Gravity.START);
        rlp.setMargins(dp(16), 0, 0, dp(30));
        root.addView(rangeToggle, rlp);
        fullRange = getSharedPreferences("camera", MODE_PRIVATE).getBoolean("fullRange", false);
        quality = getSharedPreferences("camera", MODE_PRIVATE).getInt("quality", 88);
        sizeChoice = getSharedPreferences("camera", MODE_PRIVATE).getInt("size", 0);
        updateRangeLabel();
        rangeToggle.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                fullRange = !fullRange;
                getSharedPreferences("camera", MODE_PRIVATE).edit()
                        .putBoolean("fullRange", fullRange).apply();
                updateRangeLabel();
            }
        });

        // Resolution and quality decide what a photo looks like, so they are chips on
        // the screen rather than buried settings. Both are remembered; changing the
        // resolution reopens the camera because the still size is fixed per session.
        sizeToggle = chip();
        FrameLayout.LayoutParams sizeLp = new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT,
                Gravity.TOP | Gravity.START);
        sizeLp.setMargins(dp(16), dp(64), 0, 0);
        root.addView(sizeToggle, sizeLp);
        sizeToggle.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                sizeChoice = (sizeChoice + 1) % SIZE_CHOICES.length;
                getSharedPreferences("camera", MODE_PRIVATE).edit()
                        .putInt("size", sizeChoice).apply();
                updateSettingsLabels();
                closeCamera();
                openCamera();
            }
        });

        qualityToggle = chip();
        FrameLayout.LayoutParams qualityLp = new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT,
                Gravity.TOP | Gravity.START);
        qualityLp.setMargins(dp(16), dp(104), 0, 0);
        root.addView(qualityToggle, qualityLp);
        qualityToggle.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                int[] steps = { 75, 85, 88, 95 };
                int at = 0;
                for (int i = 0; i < steps.length; i++) {
                    if (steps[i] == quality) {
                        at = i;
                    }
                }
                quality = steps[(at + 1) % steps.length];
                getSharedPreferences("camera", MODE_PRIVATE).edit()
                        .putInt("quality", quality).apply();
                updateSettingsLabels();
                updateRangeLabel();
            }
        });
        updateSettingsLabels();

        worker = new HandlerThread("mtc-camera");
        worker.start();
        cameraHandler = new Handler(worker.getLooper());

        preview.setSurfaceTextureListener(new TextureView.SurfaceTextureListener() {
            @Override
            public void onSurfaceTextureAvailable(SurfaceTexture surface, int width, int height) {
                openCamera();
            }

            @Override
            public void onSurfaceTextureSizeChanged(SurfaceTexture surface, int width, int height) {
                configureTransform();
            }

            @Override
            public boolean onSurfaceTextureDestroyed(SurfaceTexture surface) {
                return true;
            }

            @Override
            public void onSurfaceTextureUpdated(SurfaceTexture surface) {
            }
        });

        if (checkSelfPermission(Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(new String[]{Manifest.permission.CAMERA}, REQ_CAMERA);
        }
    }

    @Override
    public void onRequestPermissionsResult(int code, String[] permissions, int[] results) {
        super.onRequestPermissionsResult(code, permissions, results);
        if (code == REQ_CAMERA
                && (results.length == 0 || results[0] != PackageManager.PERMISSION_GRANTED)) {
            Toast.makeText(this, "没有相机权限", Toast.LENGTH_LONG).show();
            finish();
        }
    }

    private void openCamera() {
        if (checkSelfPermission(Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            return;
        }
        CameraManager manager = (CameraManager) getSystemService(Context.CAMERA_SERVICE);
        try {
            cameraId = pickCamera(manager, front);
            if (cameraId == null) {
                Toast.makeText(this, "找不到相机", Toast.LENGTH_LONG).show();
                return;
            }
            CameraCharacteristics chars = manager.getCameraCharacteristics(cameraId);
            StreamConfigurationMap map =
                    chars.get(CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP);
            stillSize = pickStillSize(map);
            previewSize = pickPreviewSize(map, stillSize);
            Integer sensor = chars.get(CameraCharacteristics.SENSOR_ORIENTATION);
            sensorOrientation = sensor == null ? 90 : sensor.intValue();
            stills = ImageReader.newInstance(stillSize.getWidth(), stillSize.getHeight(),
                    ImageFormat.YUV_420_888, 2);
            stills.setOnImageAvailableListener(new ImageReader.OnImageAvailableListener() {
                @Override
                public void onImageAvailable(ImageReader reader) {
                    onStill(reader);
                }
            }, cameraHandler);

            manager.openCamera(cameraId, new CameraDevice.StateCallback() {
                @Override
                public void onOpened(CameraDevice camera) {
                    device = camera;
                    startPreview();
                }

                @Override
                public void onDisconnected(CameraDevice camera) {
                    camera.close();
                    device = null;
                }

                @Override
                public void onError(CameraDevice camera, final int error) {
                    camera.close();
                    device = null;
                    ui.post(new Runnable() {
                        @Override
                        public void run() {
                            hud.setText("相机出错 " + error);
                        }
                    });
                }
            }, cameraHandler);
        } catch (CameraAccessException e) {
            Toast.makeText(this, "打不开相机：" + e.getMessage(), Toast.LENGTH_LONG).show();
        } catch (SecurityException e) {
            Toast.makeText(this, "没有相机权限", Toast.LENGTH_LONG).show();
        }
    }

    /** The camera pointing the requested way, or whatever exists. */
    private String pickCamera(CameraManager manager, boolean wantFront)
            throws CameraAccessException {
        int wanted = wantFront
                ? CameraCharacteristics.LENS_FACING_FRONT
                : CameraCharacteristics.LENS_FACING_BACK;
        String fallback = null;
        String[] ids = manager.getCameraIdList();
        for (int i = 0; i < ids.length; i++) {
            String id = ids[i];
            CameraCharacteristics chars = manager.getCameraCharacteristics(id);
            Integer facing = chars.get(CameraCharacteristics.LENS_FACING);
            if (facing != null && facing.intValue() == wanted) {
                return id;
            }
            if (fallback == null) {
                fallback = id;
            }
        }
        return fallback;
    }

    /**
     * The size the preview surface is actually created with.
     *
     * <p>This used to be the still size, up to 3264x2448. A camera does not offer
     * that as a preview size, so the preview stream never produced a frame: the
     * screen stayed black while the shutter still worked, because the still goes
     * through its own ImageReader. Preview and still are separate streams and are
     * allowed to differ.
     */
    private Size pickPreviewSize(StreamConfigurationMap map, Size still) {
        Size[] sizes = map == null ? null : map.getOutputSizes(SurfaceTexture.class);
        if (sizes == null || sizes.length == 0) {
            return new Size(1280, 720);
        }
        double target = (double) still.getWidth() / still.getHeight();
        Size best = null;
        Size closest = sizes[0];
        double closestDelta = Double.MAX_VALUE;
        for (int i = 0; i < sizes.length; i++) {
            Size s = sizes[i];
            long pixels = (long) s.getWidth() * s.getHeight();
            double aspect = (double) s.getWidth() / s.getHeight();
            double delta = Math.abs(aspect - target);
            if (delta < closestDelta) {
                closestDelta = delta;
                closest = s;
            }
            // Same shape as the photo, and no bigger than 1080p: full HD is plenty
            // for a preview and keeps the texture cheap.
            if (delta < 0.02 && pixels <= 1920 * 1088) {
                if (best == null || pixels > (long) best.getWidth() * best.getHeight()) {
                    best = s;
                }
            }
        }
        return best != null ? best : closest;
    }

    /** Pixel cap for the still, from the current resolution setting. */
    private long maxPixels() {
        int choice = Math.max(0, Math.min(SIZE_CHOICES.length - 1, sizeChoice));
        return SIZE_CHOICES[choice][0];
    }

    private Size pickStillSize(StreamConfigurationMap map) {
        Size[] sizes = map == null ? null : map.getOutputSizes(ImageFormat.YUV_420_888);
        if (sizes == null || sizes.length == 0) {
            return new Size(1920, 1080);
        }
        List<Size> candidates = new ArrayList<Size>(Arrays.asList(sizes));
        Collections.sort(candidates, new Comparator<Size>() {
            @Override
            public int compare(Size a, Size b) {
                return Long.compare((long) b.getWidth() * b.getHeight(),
                        (long) a.getWidth() * a.getHeight());
            }
        });
        for (int i = 0; i < candidates.size(); i++) {
            Size s = candidates.get(i);
            if ((long) s.getWidth() * s.getHeight() <= maxPixels()) {
                return s;
            }
        }
        return candidates.get(candidates.size() - 1);
    }

    /** Clockwise quarter turn that puts a still upright, in degrees. */
    private int stillRotation() {
        int display = 0;
        switch (getWindowManager().getDefaultDisplay().getRotation()) {
            case Surface.ROTATION_90:
                display = 90;
                break;
            case Surface.ROTATION_180:
                display = 180;
                break;
            case Surface.ROTATION_270:
                display = 270;
                break;
            default:
                display = 0;
                break;
        }
        // The sensor is mounted at an angle. A back camera needs that angle
        // clockwise; a front camera is mounted the other way round and then
        // mirrored, so its turn runs the opposite way.
        if (front) {
            return ((360 - sensorOrientation) + display) % 360;
        }
        return ((sensorOrientation - display) + 360) % 360;
    }

    /**
     * Rotate and centre-crop the preview so it is upright and fills the screen.
     *
     * <p>The texture is the sensor's landscape buffer; a portrait phone needs a
     * quarter turn and then a scale that keeps the middle rather than stretching.
     */
    /** A small translucent pill, the same shape as the colour-range switch. */
    private TextView chip() {
        TextView view = new TextView(this);
        view.setTextColor(0xFFEDEDF0);
        view.setTextSize(12);
        view.setPadding(dp(16), dp(10), dp(16), dp(10));
        view.setBackgroundColor(0x66000000);
        return view;
    }

    private String sizeLabel() {
        int choice = Math.max(0, Math.min(SIZE_CHOICES.length - 1, sizeChoice));
        switch (choice) {
            case 1:
                return "分辨率：12MP";
            case 2:
                return "分辨率：最大";
            default:
                return "分辨率：8MP";
        }
    }

    private void updateSettingsLabels() {
        if (sizeToggle != null) {
            sizeToggle.setText(sizeLabel());
        }
        if (qualityToggle != null) {
            qualityToggle.setText("质量：q" + quality);
        }
    }

    private void updateRangeLabel() {
        rangeToggle.setText(fullRange ? "色彩范围：全 (0-255)" : "色彩范围：有限 (16-235)");
    }

    private void configureTransform() {
        if (preview.getWidth() == 0 || preview.getHeight() == 0) {
            return;
        }
        // Turn by the same angle the still gets, measured from the sensor, so the
        // preview and the saved picture cannot disagree.
        int deg = stillRotation();
        float vw = preview.getWidth();
        float vh = preview.getHeight();
        boolean swapped = (deg % 180) != 0;

        // The texture is the sensor's landscape buffer; after the turn its width and
        // height swap. Centre that rectangle on the screen, then let a uniform scale
        // and the rotation about the screen centre do the rest - the recipe from the
        // Camera2 sample, with the quarter turn taken from the sensor instead of
        // from a hardcoded 90.
        android.graphics.RectF viewRect = new android.graphics.RectF(0, 0, vw, vh);
        android.graphics.RectF bufferRect = new android.graphics.RectF(0, 0,
                swapped ? previewSize.getHeight() : previewSize.getWidth(),
                swapped ? previewSize.getWidth() : previewSize.getHeight());
        float cx = viewRect.centerX();
        float cy = viewRect.centerY();
        bufferRect.offset(cx - bufferRect.centerX(), cy - bufferRect.centerY());

        android.graphics.Matrix matrix = new android.graphics.Matrix();
        matrix.setRectToRect(viewRect, bufferRect, android.graphics.Matrix.ScaleToFit.FILL);
        float scale = Math.max(vh / bufferRect.height(), vw / bufferRect.width());
        matrix.postScale(scale, scale, cx, cy);
        matrix.postRotate(deg, cx, cy);
        preview.setTransform(matrix);
    }

    private void startPreview() {
        if (device == null || !preview.isAvailable()) {
            return;
        }
        try {
            SurfaceTexture texture = preview.getSurfaceTexture();
            texture.setDefaultBufferSize(previewSize.getWidth(), previewSize.getHeight());
            Surface surface = new Surface(texture);
            final CaptureRequest.Builder builder =
                    device.createCaptureRequest(CameraDevice.TEMPLATE_PREVIEW);
            builder.addTarget(surface);
            builder.set(CaptureRequest.CONTROL_MODE, CaptureRequest.CONTROL_MODE_AUTO);
            // Ask for continuous autofocus and auto exposure explicitly. A device
            // that is never told to focus may keep the lens where it was, and every
            // picture comes out soft - so this is not a nicety.
            builder.set(CaptureRequest.CONTROL_AF_MODE,
                    CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
            builder.set(CaptureRequest.CONTROL_AE_MODE, CaptureRequest.CONTROL_AE_MODE_ON);

            List<Surface> targets = new ArrayList<Surface>();
            targets.add(surface);
            targets.add(stills.getSurface());
            device.createCaptureSession(targets, new CameraCaptureSession.StateCallback() {
                @Override
                public void onConfigured(CameraCaptureSession configured) {
                    session = configured;
                    try {
                        session.setRepeatingRequest(builder.build(), null, cameraHandler);
                        preview.post(new Runnable() {
                            @Override
                            public void run() {
                                configureTransform();
                            }
                        });
                        ui.post(new Runnable() {
                            @Override
                            public void run() {
                                hud.setText(String.format(Locale.US,
                                        "拍 %d×%d · 预览 %d×%d · q%d · 传感器 %d° 转 %d°",
                                        stillSize.getWidth(), stillSize.getHeight(),
                                        previewSize.getWidth(), previewSize.getHeight(), quality,
                                        sensorOrientation, stillRotation()));
                            }
                        });
                    } catch (CameraAccessException e) {
                        final String msg = e.getMessage();
                        ui.post(new Runnable() {
                            @Override
                            public void run() {
                                hud.setText("预览失败：" + msg);
                            }
                        });
                    }
                }

                @Override
                public void onConfigureFailed(CameraCaptureSession failed) {
                    ui.post(new Runnable() {
                        @Override
                        public void run() {
                            hud.setText("预览配置失败");
                        }
                    });
                }
            }, cameraHandler);
        } catch (CameraAccessException e) {
            final String msg = e.getMessage();
            ui.post(new Runnable() {
                @Override
                public void run() {
                    hud.setText("预览出错：" + msg);
                }
            });
        }
    }

    private void capture() {
        if (busy || device == null || session == null) {
            return;
        }
        busy = true;
        hud.setText("拍摄中…");
        try {
            CaptureRequest.Builder builder =
                    device.createCaptureRequest(CameraDevice.TEMPLATE_STILL_CAPTURE);
            builder.addTarget(stills.getSurface());
            builder.set(CaptureRequest.CONTROL_MODE, CaptureRequest.CONTROL_MODE_AUTO);
            // The preview runs continuous autofocus; carry it into the still so the
            // frame that gets compressed is a focused one. Without an AF mode some
            // devices never move the lens, and every photo is soft.
            builder.set(CaptureRequest.CONTROL_AF_MODE,
                    CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
            builder.set(CaptureRequest.CONTROL_AE_MODE, CaptureRequest.CONTROL_AE_MODE_ON);
            session.capture(builder.build(), null, cameraHandler);
        } catch (CameraAccessException e) {
            busy = false;
            hud.setText("拍摄失败：" + e.getMessage());
        }
    }

    /** ImageReader callback: copy the planes out, encode, save. */
    private void onStill(ImageReader reader) {
        Image image = reader.acquireNextImage();
        if (image == null) {
            return;
        }
        if (image.getFormat() != ImageFormat.YUV_420_888) {
            image.close();
            return;
        }
        final long start = System.currentTimeMillis();
        final int w = image.getWidth();
        final int h = image.getHeight();
        byte[] blob = null;
        try {
            Image.Plane[] planes = image.getPlanes();
            byte[] y = copy(planes[0]);
            byte[] u = copy(planes[1]);
            byte[] v = copy(planes[2]);
            int yStride = planes[0].getRowStride();
            int uvStride = planes[1].getRowStride();
            int uvPixel = planes[1].getPixelStride();
            blob = Mtc.encodeYuv(y, u, v, w, h, yStride, uvStride, uvPixel,
                    quality, fullRange, stillRotation(), front);
        } catch (RuntimeException e) {
            blob = null;
        } finally {
            image.close();
        }
        final byte[] encoded = blob;
        final long ms = System.currentTimeMillis() - start;
        if (encoded == null) {
            ui.post(new Runnable() {
                @Override
                public void run() {
                    busy = false;
                    hud.setText("编码失败");
                }
            });
            return;
        }
        final File out = Store.nextPhoto(this);
        final boolean ok = Store.write(out, encoded);
        if (ok) {
            // Decoding 12 MP to fill a grid cell is expensive; pay it once, off
            // the shutter path, and cache the result next to the picture.
            cameraHandler.post(new Runnable() {
                @Override
                public void run() {
                    Store.writeThumbnail(out, 384);
                }
            });
        }
        ui.post(new Runnable() {
            @Override
            public void run() {
                busy = false;
                if (ok) {
                    hud.setText(String.format(Locale.US, "%d×%d · %s · %d ms · q%d",
                            w, h, Store.human(encoded.length), ms, quality));
                } else {
                    hud.setText("写文件失败");
                }
                Toast.makeText(CameraActivity.this,
                        ok ? "已保存 " + Store.human(encoded.length) : "保存失败",
                        Toast.LENGTH_SHORT).show();
            }
        });
    }

    private static byte[] copy(Image.Plane plane) {
        ByteBuffer buffer = plane.getBuffer();
        buffer.rewind();
        byte[] out = new byte[buffer.remaining()];
        buffer.get(out);
        return out;
    }

    @Override
    protected void onResume() {
        super.onResume();
        if (device == null && preview.isAvailable()) {
            openCamera();
        }
    }

    @Override
    protected void onPause() {
        super.onPause();
        closeCamera();
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        closeCamera();
        if (worker != null) {
            worker.quitSafely();
        }
    }

    private void closeCamera() {
        if (session != null) {
            session.close();
            session = null;
        }
        if (device != null) {
            device.close();
            device = null;
        }
        if (stills != null) {
            stills.close();
            stills = null;
        }
    }

    private int dp(int v) {
        return Math.round(v * getResources().getDisplayMetrics().density);
    }
}

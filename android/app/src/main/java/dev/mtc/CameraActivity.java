package dev.mtc;

import android.Manifest;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.ContentResolver;
import android.content.ContentValues;
import android.content.Context;
import android.content.SharedPreferences;
import android.content.pm.PackageManager;
import android.graphics.Color;
import android.graphics.SurfaceTexture;
import android.graphics.drawable.GradientDrawable;
import android.hardware.camera2.CameraAccessException;
import android.hardware.camera2.CameraCaptureSession;
import android.hardware.camera2.CameraCharacteristics;
import android.hardware.camera2.CameraDevice;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.CaptureRequest;
import android.hardware.camera2.params.StreamConfigurationMap;
import android.media.Image;
import android.media.ImageReader;
import android.media.MediaRecorder;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.HandlerThread;
import android.os.Looper;
import android.os.ParcelFileDescriptor;
import android.provider.MediaStore;
import android.util.Size;
import android.view.Gravity;
import android.view.Surface;
import android.view.TextureView;
import android.view.View;
import android.view.ViewGroup;
import android.view.WindowManager;
import android.widget.FrameLayout;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;
import android.widget.Toast;

import java.io.File;
import java.io.FileOutputStream;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.Comparator;
import java.util.Date;
import java.util.List;
import java.util.Locale;

/**
 * The camera: a photo mode that writes .mtc, and a video mode that writes MP4.
 *
 * <p>Both land where other apps can see them - DCIM/MTC for pictures, Movies/MTC for
 * video - through MediaStore, so nothing is trapped in the app's private folder.
 *
 * <p>The screen is a full-bleed preview and a bottom bar: photo, shutter, video, and a
 * settings sheet. Everything adjustable lives in that sheet, because floating chips over
 * the picture are what a viewfinder does not need.
 */
public class CameraActivity extends Activity {
    private static final int REQ_CAMERA = 1;

    private static final int[][] SIZE_CHOICES = {
            {3264 * 2448, 0},        // 8 MP, the default
            {4080 * 3060, 1},        // 12 MP
            {Integer.MAX_VALUE, 2},  // whatever the sensor offers
    };
    private static final int[] QUALITY_STEPS = { 75, 85, 88, 95 };

    private static final String SP = "camera";
    private static final String PUBLIC_ALBUM = "DCIM/MTC";
    private static final String PUBLIC_VIDEO = "Movies/MTC";

    private HandlerThread worker;
    private Handler cameraHandler;
    private final Handler ui = new Handler(Looper.getMainLooper());

    private TextureView preview;
    private FrameLayout root;
    private TextView modePhoto;
    private TextView modeVideo;
    private TextView recordTime;
    private View shutterInner;
    private TextView settingsButton;

    private CameraDevice device;
    private CameraCaptureSession session;
    private ImageReader stills;
    private Size stillSize = new Size(3264, 2448);
    private Size previewSize = new Size(1440, 1080);
    private String cameraId;
    private boolean busy;

    private int quality = 88;
    private int sizeChoice;
    private boolean fullRange;
    private boolean front;
    private int rotationOverride = -1;
    private int sensorOrientation = 90;

    private boolean videoMode;
    private MediaRecorder recorder;
    private ParcelFileDescriptor videoFile;
    private Uri videoUri;
    private Surface recorderSurface;
    private long recordStarted;
    private boolean recording;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        // The thread has to exist before the surface listener can fire.
        worker = new HandlerThread("mtc-camera");
        worker.start();
        cameraHandler = new Handler(worker.getLooper());
        loadSettings();
        buildUi();
    }

    private void loadSettings() {
        SharedPreferences sp = getSharedPreferences(SP, MODE_PRIVATE);
        fullRange = sp.getBoolean("fullRange", false);
        quality = sp.getInt("quality", 88);
        sizeChoice = sp.getInt("size", 0);
        rotationOverride = sp.getInt("rotation", -1);
    }

    private void save(String key, int value) {
        getSharedPreferences(SP, MODE_PRIVATE).edit().putInt(key, value).apply();
    }

    // ───────────────────────────── interface ─────────────────────────────

    private int dp(float value) {
        return Math.round(value * getResources().getDisplayMetrics().density);
    }

    private void buildUi() {
        root = new FrameLayout(this);
        root.setBackgroundColor(Color.BLACK);

        preview = new TextureView(this);
        root.addView(preview, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
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

        TextView close = new TextView(this);
        close.setText("✕");
        close.setTextColor(Color.WHITE);
        close.setTextSize(20);
        close.setGravity(Gravity.CENTER);
        FrameLayout.LayoutParams closeLp = new FrameLayout.LayoutParams(dp(48), dp(48),
                Gravity.TOP | Gravity.END);
        closeLp.setMargins(0, dp(12), dp(12), 0);
        root.addView(close, closeLp);
        close.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                finish();
            }
        });

        recordTime = new TextView(this);
        recordTime.setTextColor(Color.WHITE);
        recordTime.setTextSize(13);
        recordTime.setPadding(dp(10), dp(4), dp(10), dp(4));
        recordTime.setBackgroundColor(0x99000000);
        recordTime.setVisibility(View.GONE);
        FrameLayout.LayoutParams recLp = new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT,
                Gravity.TOP | Gravity.CENTER_HORIZONTAL);
        recLp.setMargins(0, dp(16), 0, 0);
        root.addView(recordTime, recLp);

        LinearLayout bar = new LinearLayout(this);
        bar.setOrientation(LinearLayout.HORIZONTAL);
        bar.setGravity(Gravity.CENTER_VERTICAL);
        bar.setBackgroundColor(0xB3000000);
        FrameLayout.LayoutParams barLp = new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, dp(108), Gravity.BOTTOM);
        root.addView(bar, barLp);

        modePhoto = new TextView(this);
        modePhoto.setText("拍照");
        modePhoto.setTextSize(14);
        modePhoto.setGravity(Gravity.CENTER);
        bar.addView(modePhoto, new LinearLayout.LayoutParams(0,
                ViewGroup.LayoutParams.MATCH_PARENT, 1f));

        modeVideo = new TextView(this);
        modeVideo.setText("视频");
        modeVideo.setTextSize(14);
        modeVideo.setGravity(Gravity.CENTER);
        bar.addView(modeVideo, new LinearLayout.LayoutParams(0,
                ViewGroup.LayoutParams.MATCH_PARENT, 1f));

        FrameLayout shutterHolder = new FrameLayout(this);
        bar.addView(shutterHolder, new LinearLayout.LayoutParams(dp(110),
                ViewGroup.LayoutParams.MATCH_PARENT));

        View shutterOuter = new View(this);
        GradientDrawable ring = new GradientDrawable();
        ring.setShape(GradientDrawable.OVAL);
        ring.setColor(Color.TRANSPARENT);
        ring.setStroke(dp(4), Color.WHITE);
        shutterOuter.setBackground(ring);
        shutterHolder.addView(shutterOuter, new FrameLayout.LayoutParams(dp(72), dp(72),
                Gravity.CENTER));

        shutterInner = new View(this);
        GradientDrawable dot = new GradientDrawable();
        dot.setShape(GradientDrawable.OVAL);
        dot.setColor(Color.WHITE);
        shutterInner.setBackground(dot);
        shutterHolder.addView(shutterInner, new FrameLayout.LayoutParams(dp(56), dp(56),
                Gravity.CENTER));

        shutterHolder.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                if (videoMode) {
                    toggleRecording();
                } else {
                    capture();
                }
            }
        });

        TextView gallery = new TextView(this);
        gallery.setText("相册");
        gallery.setTextSize(13);
        gallery.setGravity(Gravity.CENTER);
        gallery.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                finish();
            }
        });
        bar.addView(gallery, new LinearLayout.LayoutParams(0,
                ViewGroup.LayoutParams.MATCH_PARENT, 1f));

        settingsButton = new TextView(this);
        settingsButton.setText("⚙");
        settingsButton.setTextSize(18);
        settingsButton.setGravity(Gravity.CENTER);
        bar.addView(settingsButton, new LinearLayout.LayoutParams(dp(56),
                ViewGroup.LayoutParams.MATCH_PARENT));
        settingsButton.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                showSettings();
            }
        });

        modePhoto.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                setVideoMode(false);
            }
        });
        modeVideo.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                setVideoMode(true);
            }
        });

        setContentView(root);
        updateModeUi();
    }

    private void setVideoMode(boolean video) {
        if (recording) {
            return;
        }
        videoMode = video;
        updateModeUi();
        closeCamera();
        openCamera();
    }

    private void updateModeUi() {
        modePhoto.setTextColor(videoMode ? 0xFF9A9AA2 : Color.WHITE);
        modeVideo.setTextColor(videoMode ? Color.WHITE : 0xFF9A9AA2);
        GradientDrawable dot = new GradientDrawable();
        dot.setShape(GradientDrawable.OVAL);
        dot.setColor(recording ? Color.parseColor("#FF3B30") : Color.WHITE);
        shutterInner.setBackground(dot);
    }

    // ───────────────────────────── settings ─────────────────────────────

    private void showSettings() {
        LinearLayout sheet = new LinearLayout(this);
        sheet.setOrientation(LinearLayout.VERTICAL);
        sheet.setPadding(dp(20), dp(8), dp(20), dp(8));

        final TextView resolution = settingRow(sheet);
        final TextView qualityRow = settingRow(sheet);
        final TextView rotation = settingRow(sheet);
        final TextView range = settingRow(sheet);
        final TextView diagnostics = new TextView(this);
        diagnostics.setTextSize(11.5f);
        diagnostics.setTextColor(0xFF666670);
        diagnostics.setPadding(0, dp(12), 0, 0);
        sheet.addView(diagnostics);

        final Runnable refresh = new Runnable() {
            @Override
            public void run() {
                resolution.setText("分辨率：" + sizeLabel() + "（点一下换一档）");
                qualityRow.setText("质量：q" + quality);
                rotation.setText(rotationOverride < 0
                        ? "方向：自动（传感器 " + sensorOrientation + "°）"
                        : "方向：固定 " + rotationOverride + "°");
                range.setText(fullRange ? "色彩范围：全 (0-255)" : "色彩范围：有限 (16-235)");
                diagnostics.setText(String.format(Locale.US,
                        "拍照 %d×%d · 预览 %d×%d · 传感器 %d° 转 %d°\n照片存到 %s，视频存到 %s",
                        stillSize.getWidth(), stillSize.getHeight(),
                        previewSize.getWidth(), previewSize.getHeight(),
                        sensorOrientation, stillRotation(), PUBLIC_ALBUM, PUBLIC_VIDEO));
            }
        };
        refresh.run();

        resolution.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                sizeChoice = (sizeChoice + 1) % SIZE_CHOICES.length;
                save("size", sizeChoice);
                refresh.run();
                closeCamera();
                openCamera();
            }
        });
        qualityRow.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                int at = 0;
                for (int i = 0; i < QUALITY_STEPS.length; i++) {
                    if (QUALITY_STEPS[i] == quality) {
                        at = i;
                    }
                }
                quality = QUALITY_STEPS[(at + 1) % QUALITY_STEPS.length];
                save("quality", quality);
                refresh.run();
            }
        });
        rotation.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                switch (rotationOverride) {
                    case -1:
                        rotationOverride = 0;
                        break;
                    case 0:
                        rotationOverride = 90;
                        break;
                    case 90:
                        rotationOverride = 180;
                        break;
                    case 180:
                        rotationOverride = 270;
                        break;
                    default:
                        rotationOverride = -1;
                        break;
                }
                save("rotation", rotationOverride);
                refresh.run();
                closeCamera();
                openCamera();
            }
        });
        range.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                fullRange = !fullRange;
                getSharedPreferences(SP, MODE_PRIVATE).edit()
                        .putBoolean("fullRange", fullRange).apply();
                refresh.run();
            }
        });

        ScrollView scroll = new ScrollView(this);
        scroll.addView(sheet);
        new AlertDialog.Builder(this)
                .setTitle("相机设置")
                .setView(scroll)
                .setPositiveButton("完成", null)
                .show();
    }

    private TextView settingRow(LinearLayout parent) {
        TextView row = new TextView(this);
        row.setTextSize(14.5f);
        row.setTextColor(0xFF1A1A1A);
        row.setPadding(0, dp(14), 0, dp(14));
        parent.addView(row);
        return row;
    }

    private String sizeLabel() {
        int choice = Math.max(0, Math.min(SIZE_CHOICES.length - 1, sizeChoice));
        switch (choice) {
            case 1:
                return "12MP";
            case 2:
                return "最大";
            default:
                return "8MP";
        }
    }

    // ───────────────────────────── camera ─────────────────────────────

    private void openCamera() {
        if (checkSelfPermission(Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            requestPermissions(new String[]{Manifest.permission.CAMERA}, REQ_CAMERA);
            return;
        }
        if (preview.getSurfaceTexture() == null) {
            return;
        }
        CameraManager manager = (CameraManager) getSystemService(Context.CAMERA_SERVICE);
        try {
            cameraId = pickCamera(manager, front);
            if (cameraId == null) {
                toast("找不到相机");
                return;
            }
            CameraCharacteristics chars = manager.getCameraCharacteristics(cameraId);
            StreamConfigurationMap map =
                    chars.get(CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP);
            stillSize = pickStillSize(map);
            previewSize = pickPreviewSize(map, stillSize);
            Integer sensor = chars.get(CameraCharacteristics.SENSOR_ORIENTATION);
            sensorOrientation = sensor == null ? 90 : sensor.intValue();

            if (stills != null) {
                stills.close();
            }
            stills = ImageReader.newInstance(stillSize.getWidth(), stillSize.getHeight(),
                    android.graphics.ImageFormat.YUV_420_888, 2);
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
                    toast("相机出错 " + error);
                }
            }, cameraHandler);
        } catch (CameraAccessException e) {
            toast("打不开相机：" + e.getMessage());
        } catch (SecurityException e) {
            toast("没有相机权限");
        }
    }

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

    private long maxPixels() {
        int choice = Math.max(0, Math.min(SIZE_CHOICES.length - 1, sizeChoice));
        return SIZE_CHOICES[choice][0];
    }

    /**
     * The still size: 4:3 first, then 16:9, then whatever is largest under the cap.
     *
     * <p>Area alone is not enough - one device offered a 4096x1844 stream as its largest
     * size under 8 MP, which is a 2.2:1 strip rather than a photograph.
     */
    private Size pickStillSize(StreamConfigurationMap map) {
        Size[] sizes = map == null ? null
                : map.getOutputSizes(android.graphics.ImageFormat.YUV_420_888);
        if (sizes == null || sizes.length == 0) {
            return new Size(1920, 1080);
        }
        Size squarish = pickByAspect(sizes, 4.0 / 3.0);
        if (squarish != null) {
            return squarish;
        }
        Size wide = pickByAspect(sizes, 16.0 / 9.0);
        if (wide != null) {
            return wide;
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

    private Size pickByAspect(Size[] sizes, double want) {
        Size best = null;
        long bestPixels = 0;
        for (int i = 0; i < sizes.length; i++) {
            Size s = sizes[i];
            double aspect = (double) s.getWidth() / s.getHeight();
            if (Math.abs(aspect - want) > 0.03) {
                continue;
            }
            long pixels = (long) s.getWidth() * s.getHeight();
            if (pixels > maxPixels() || pixels <= bestPixels) {
                continue;
            }
            best = s;
            bestPixels = pixels;
        }
        return best;
    }

    /** The preview surface size, chosen to match the photo's shape. */
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
            if (delta < 0.02 && pixels <= 1920 * 1088) {
                if (best == null || pixels > (long) best.getWidth() * best.getHeight()) {
                    best = s;
                }
            }
        }
        return best != null ? best : closest;
    }

    /** Clockwise quarter turn that puts a picture upright, in degrees. */
    private int stillRotation() {
        if (rotationOverride >= 0) {
            return rotationOverride;
        }
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
        if (front) {
            return ((360 - sensorOrientation) + display) % 360;
        }
        return ((sensorOrientation - display) + 360) % 360;
    }

    /**
     * Turn the preview by the same angle the photo gets, and scale it uniformly.
     *
     * <p>Uniform is the point: an earlier version scaled the two axes independently to
     * fill the screen, which stretches the picture whenever the view's shape differs from
     * the stream's. This centres a cover-scaled, rotated rectangle instead, so the worst
     * case is cropping - never distortion.
     */
    private void configureTransform() {
        if (preview.getWidth() == 0 || preview.getHeight() == 0) {
            return;
        }
        int deg = stillRotation();
        float vw = preview.getWidth();
        float vh = preview.getHeight();
        boolean swapped = (deg % 180) != 0;
        float bw = swapped ? previewSize.getHeight() : previewSize.getWidth();
        float bh = swapped ? previewSize.getWidth() : previewSize.getHeight();
        float scale = Math.max(vw / bw, vh / bh);

        android.graphics.Matrix matrix = new android.graphics.Matrix();
        matrix.postRotate(deg);
        matrix.postScale(scale, scale);
        matrix.postTranslate((vw - bw * scale) / 2f, (vh - bh * scale) / 2f);
        preview.setTransform(matrix);
    }

    private void startPreview() {
        if (device == null || preview.getSurfaceTexture() == null) {
            return;
        }
        try {
            SurfaceTexture texture = preview.getSurfaceTexture();
            texture.setDefaultBufferSize(previewSize.getWidth(), previewSize.getHeight());
            Surface surface = new Surface(texture);
            CaptureRequest.Builder builder =
                    device.createCaptureRequest(CameraDevice.TEMPLATE_PREVIEW);
            builder.addTarget(surface);
            builder.set(CaptureRequest.CONTROL_MODE, CaptureRequest.CONTROL_MODE_AUTO);
            builder.set(CaptureRequest.CONTROL_AF_MODE,
                    CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
            builder.set(CaptureRequest.CONTROL_AE_MODE, CaptureRequest.CONTROL_AE_MODE_ON);

            List<Surface> targets = new ArrayList<Surface>();
            targets.add(surface);
            if (!videoMode) {
                targets.add(stills.getSurface());
            }
            final CaptureRequest request = builder.build();
            device.createCaptureSession(targets, new CameraCaptureSession.StateCallback() {
                @Override
                public void onConfigured(CameraCaptureSession configured) {
                    session = configured;
                    try {
                        session.setRepeatingRequest(request, null, cameraHandler);
                        preview.post(new Runnable() {
                            @Override
                            public void run() {
                                configureTransform();
                            }
                        });
                    } catch (CameraAccessException e) {
                        toast("预览失败：" + e.getMessage());
                    }
                }

                @Override
                public void onConfigureFailed(CameraCaptureSession failed) {
                    toast("预览配置失败");
                }
            }, cameraHandler);
        } catch (CameraAccessException e) {
            toast("预览出错：" + e.getMessage());
        }
    }

    private void closeCamera() {
        closeSession();
        if (device != null) {
            device.close();
            device = null;
        }
    }

    @Override
    protected void onDestroy() {
        stopRecording(false);
        closeCamera();
        if (stills != null) {
            stills.close();
            stills = null;
        }
        if (worker != null) {
            worker.quitSafely();
        }
        super.onDestroy();
    }

    // ───────────────────────────── photo ─────────────────────────────

    private void capture() {
        if (device == null || session == null || busy) {
            return;
        }
        busy = true;
        toast("拍摄中…");
        try {
            CaptureRequest.Builder builder =
                    device.createCaptureRequest(CameraDevice.TEMPLATE_STILL_CAPTURE);
            builder.addTarget(stills.getSurface());
            builder.set(CaptureRequest.CONTROL_MODE, CaptureRequest.CONTROL_MODE_AUTO);
            builder.set(CaptureRequest.CONTROL_AF_MODE,
                    CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
            builder.set(CaptureRequest.CONTROL_AE_MODE, CaptureRequest.CONTROL_AE_MODE_ON);
            session.capture(builder.build(), null, cameraHandler);
        } catch (CameraAccessException e) {
            busy = false;
            toast("拍摄失败：" + e.getMessage());
        }
    }

    /** Encode the frame and hand it to the gallery, in DCIM/MTC. */
    private void onStill(ImageReader reader) {
        Image image = null;
        try {
            image = reader.acquireNextImage();
            if (image == null) {
                return;
            }
            Image.Plane[] planes = image.getPlanes();
            ByteBuffer yBuffer = planes[0].getBuffer();
            ByteBuffer uBuffer = planes[1].getBuffer();
            ByteBuffer vBuffer = planes[2].getBuffer();
            byte[] y = new byte[yBuffer.remaining()];
            byte[] u = new byte[uBuffer.remaining()];
            byte[] v = new byte[vBuffer.remaining()];
            yBuffer.get(y);
            uBuffer.get(u);
            vBuffer.get(v);

            int yStride = planes[0].getRowStride();
            int uvStride = planes[1].getRowStride();
            int uvPixel = planes[1].getPixelStride();

            long t0 = System.currentTimeMillis();
            final byte[] encoded = Mtc.encodeYuv(y, u, v,
                    image.getWidth(), image.getHeight(), yStride, uvStride, uvPixel,
                    quality, fullRange, stillRotation(), front);
            final long ms = System.currentTimeMillis() - t0;
            final Size size = stillSize;
            image.close();
            image = null;

            if (encoded == null) {
                toast("编码失败");
                return;
            }
            final String name = "IMG_" + new SimpleDateFormat("yyyyMMdd_HHmmss",
                    Locale.US).format(new Date()) + ".mtc";
            final String where = savePicture(name, encoded);
            ui.post(new Runnable() {
                @Override
                public void run() {
                    toast(where == null
                            ? "保存失败"
                            : "已保存 " + Store.human(encoded.length) + " · " + ms + " ms · "
                              + size.getWidth() + "×" + size.getHeight());
                }
            });
        } catch (Exception e) {
            final String message = e.getMessage();
            ui.post(new Runnable() {
                @Override
                public void run() {
                    toast("拍摄出错：" + message);
                }
            });
        } finally {
            if (image != null) {
                image.close();
            }
            ui.post(new Runnable() {
                @Override
                public void run() {
                    busy = false;
                }
            });
        }
    }

    /**
     * Write a picture into the public gallery, falling back to the app's own folder.
     *
     * <p>DCIM/MTC instead of the private directory: a photo nobody else can open is not a
     * photo. MediaStore needs no storage permission for a file the app itself creates.
     */
    private String savePicture(String name, byte[] bytes) {
        // Two copies on purpose: the public one is what every other app and a PC can
        // open, the private one is what this app's gallery browses. They are the same
        // bytes and a .mtc is tens of kilobytes, so the duplication is cheap next to
        // having a photo that only one app can see.
        writePrivate(name, bytes);
        if (Build.VERSION.SDK_INT >= 29) {
            try {
                ContentValues values = new ContentValues();
                values.put(MediaStore.MediaColumns.DISPLAY_NAME, name);
                values.put(MediaStore.MediaColumns.MIME_TYPE, "application/octet-stream");
                values.put(MediaStore.MediaColumns.RELATIVE_PATH, PUBLIC_ALBUM);
                values.put(MediaStore.MediaColumns.IS_PENDING, 1);
                ContentResolver resolver = getContentResolver();
                Uri target = resolver.insert(
                        MediaStore.Files.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY), values);
                if (target != null) {
                    OutputStream out = resolver.openOutputStream(target);
                    try {
                        out.write(bytes);
                    } finally {
                        out.close();
                    }
                    values.clear();
                    values.put(MediaStore.MediaColumns.IS_PENDING, 0);
                    resolver.update(target, values, null, null);
                    return PUBLIC_ALBUM + "/" + name;
                }
            } catch (Exception ignored) {
                // fall through to the app folder
            }
        }
        File out = new File(Store.dir(this), name);
        return writePrivate(name, bytes) ? out.getAbsolutePath() : null;
    }

    private boolean writePrivate(String name, byte[] bytes) {
        try {
            FileOutputStream fos = new FileOutputStream(new File(Store.dir(this), name));
            try {
                fos.write(bytes);
            } finally {
                fos.close();
            }
            return true;
        } catch (Exception e) {
            return false;
        }
    }

    // ───────────────────────────── video ─────────────────────────────

    private void toggleRecording() {
        if (recording) {
            stopRecording(true);
        } else {
            startRecording();
        }
    }

    private void startRecording() {
        if (device == null || recording) {
            return;
        }
        final int rotation = stillRotation();
        try {
            Uri target = createVideoTarget();
            if (target == null) {
                toast("建不了视频文件");
                return;
            }
            videoUri = target;
            videoFile = getContentResolver().openFileDescriptor(target, "w");
            if (videoFile == null) {
                toast("打不开视频文件");
                return;
            }

            MediaRecorder rec = Build.VERSION.SDK_INT >= 31
                    ? new MediaRecorder(this)
                    : new MediaRecorder();
            rec.setVideoSource(MediaRecorder.VideoSource.SURFACE);
            rec.setOutputFormat(MediaRecorder.OutputFormat.MPEG_4);
            rec.setVideoEncoder(MediaRecorder.VideoEncoder.H264);
            rec.setVideoSize(previewSize.getWidth(), previewSize.getHeight());
            rec.setVideoFrameRate(30);
            rec.setVideoEncodingBitRate(bitrateFor(previewSize));
            rec.setOrientationHint(rotation);
            rec.setOutputFile(videoFile.getFileDescriptor());
            rec.prepare();
            recorder = rec;
            recorderSurface = rec.getSurface();

            final Surface previewSurface = new Surface(preview.getSurfaceTexture());
            List<Surface> targets = new ArrayList<Surface>();
            targets.add(previewSurface);
            targets.add(recorderSurface);
            closeSession();
            device.createCaptureSession(targets, new CameraCaptureSession.StateCallback() {
                @Override
                public void onConfigured(CameraCaptureSession configured) {
                    session = configured;
                    try {
                        CaptureRequest.Builder builder =
                                device.createCaptureRequest(CameraDevice.TEMPLATE_RECORD);
                        builder.addTarget(previewSurface);
                        builder.addTarget(recorderSurface);
                        builder.set(CaptureRequest.CONTROL_MODE, CaptureRequest.CONTROL_MODE_AUTO);
                        builder.set(CaptureRequest.CONTROL_AF_MODE,
                                CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
                        session.setRepeatingRequest(builder.build(), null, cameraHandler);
                        recorder.start();
                        recordStarted = System.currentTimeMillis();
                        recording = true;
                        ui.post(new Runnable() {
                            @Override
                            public void run() {
                                updateModeUi();
                                recordTime.setVisibility(View.VISIBLE);
                                tickRecordTime();
                            }
                        });
                    } catch (Exception e) {
                        stopRecording(false);
                        toast("录制失败：" + e.getMessage());
                    }
                }

                @Override
                public void onConfigureFailed(CameraCaptureSession failed) {
                    toast("录制配置失败");
                }
            }, cameraHandler);
        } catch (Exception e) {
            stopRecording(false);
            toast("录制打不开：" + e.getMessage());
        }
    }

    private int bitrateFor(Size size) {
        long pixels = (long) size.getWidth() * size.getHeight();
        // About 0.12 bits per pixel per frame at 30 fps: 1080p lands near 8 Mbit/s.
        return (int) Math.max(4_000_000L, Math.min(40_000_000L, pixels * 30 / 8));
    }

    private void tickRecordTime() {
        if (!recording) {
            return;
        }
        long seconds = (System.currentTimeMillis() - recordStarted) / 1000;
        recordTime.setText(String.format(Locale.US, "● %02d:%02d", seconds / 60, seconds % 60));
        ui.postDelayed(new Runnable() {
            @Override
            public void run() {
                tickRecordTime();
            }
        }, 500);
    }

    /** @param finalise false when a recording failed and the file should be dropped. */
    private void stopRecording(boolean finalise) {
        final boolean wasRecording = recording;
        recording = false;
        ui.post(new Runnable() {
            @Override
            public void run() {
                updateModeUi();
                recordTime.setVisibility(View.GONE);
            }
        });
        if (recorder != null) {
            if (wasRecording) {
                try {
                    recorder.stop();
                } catch (RuntimeException tooShort) {
                    finalise = false;
                }
            }
            try {
                recorder.reset();
                recorder.release();
            } catch (Exception ignored) {
            }
            recorder = null;
        }
        if (recorderSurface != null) {
            recorderSurface.release();
            recorderSurface = null;
        }
        if (videoFile != null) {
            try {
                videoFile.close();
            } catch (Exception ignored) {
            }
            videoFile = null;
        }
        final Uri uri = videoUri;
        videoUri = null;
        if (uri != null) {
            if (wasRecording && finalise) {
                try {
                    ContentValues values = new ContentValues();
                    values.put(MediaStore.MediaColumns.IS_PENDING, 0);
                    getContentResolver().update(uri, values, null, null);
                    ui.post(new Runnable() {
                        @Override
                        public void run() {
                            toast("视频已保存到 " + PUBLIC_VIDEO);
                        }
                    });
                } catch (Exception ignored) {
                }
            } else {
                try {
                    getContentResolver().delete(uri, null, null);
                } catch (Exception ignored) {
                }
            }
        }
        if (device != null) {
            closeSession();
            startPreview();
        }
    }

    private void closeSession() {
        try {
            if (session != null) {
                session.stopRepeating();
                session.close();
                session = null;
            }
        } catch (Exception ignored) {
        }
    }

    private Uri createVideoTarget() {
        if (Build.VERSION.SDK_INT < 29) {
            return null;
        }
        String name = "VID_" + new SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(new Date())
                + ".mp4";
        try {
            ContentValues values = new ContentValues();
            values.put(MediaStore.MediaColumns.DISPLAY_NAME, name);
            values.put(MediaStore.MediaColumns.MIME_TYPE, "video/mp4");
            values.put(MediaStore.MediaColumns.RELATIVE_PATH, PUBLIC_VIDEO);
            values.put(MediaStore.MediaColumns.IS_PENDING, 1);
            return getContentResolver().insert(
                    MediaStore.Video.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY), values);
        } catch (Exception e) {
            return null;
        }
    }

    // ───────────────────────────── misc ─────────────────────────────

    private void toast(String text) {
        Toast.makeText(this, text, Toast.LENGTH_SHORT).show();
    }

    @Override
    public void onRequestPermissionsResult(int request, String[] permissions, int[] granted) {
        super.onRequestPermissionsResult(request, permissions, granted);
        if (request == REQ_CAMERA && granted.length > 0
                && granted[0] == PackageManager.PERMISSION_GRANTED) {
            openCamera();
        } else {
            toast("没有相机权限，拍不了");
        }
    }
}

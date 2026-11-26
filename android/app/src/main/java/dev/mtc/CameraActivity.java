package dev.mtc;

import android.Manifest;
import android.app.Activity;
import android.app.AlertDialog;
import android.content.ContentResolver;
import android.content.ContentValues;
import android.content.Context;
import android.content.SharedPreferences;
import android.content.pm.PackageManager;
import android.graphics.BitmapFactory;
import android.graphics.Color;
import android.graphics.Rect;
import android.graphics.SurfaceTexture;
import android.graphics.drawable.GradientDrawable;
import android.hardware.camera2.CameraAccessException;
import android.hardware.camera2.CameraCaptureSession;
import android.hardware.camera2.CameraCharacteristics;
import android.hardware.camera2.CameraDevice;
import android.hardware.camera2.CameraManager;
import android.hardware.camera2.CaptureRequest;
import android.hardware.camera2.params.MeteringRectangle;
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
import android.view.MotionEvent;
import android.view.ScaleGestureDetector;
import android.view.Surface;
import android.view.TextureView;
import android.view.View;
import android.view.ViewGroup;
import android.view.WindowManager;
import android.widget.FrameLayout;
import android.widget.HorizontalScrollView;
import android.widget.ImageView;
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
 * The camera, laid out like the one on a Xiaomi phone: icons across the top, a zoom bar,
 * a mode strip, and the three buttons that matter at the bottom.
 *
 * <p>Only what Camera2 can actually do is wired up. Flash, grid, level, timer, aspect
 * ratio, zoom, tap-to-focus with exposure, burst, front/back and the gallery thumb all
 * work. The modes that need multi-frame algorithms (portrait, night, panorama, pro) are
 * on the strip but say so when tapped, rather than pretending.
 *
 * <p>Pictures are .mtc - this is the MTC codec's camera - written to DCIM/MTC, and video
 * is H.264 MP4 in Movies/MTC, both through MediaStore so other apps can see them.
 */
public class CameraActivity extends Activity implements ViewfinderOverlay.Listener {
    private static final int REQ_CAMERA = 1;

    private static final int[][] SIZE_CHOICES = {
            {3264 * 2448, 0},
            {4080 * 3060, 1},
            {Integer.MAX_VALUE, 2},
    };
    private static final int[] QUALITY_STEPS = { 75, 85, 88, 95 };
    /** 1:1, 4:3, 9:16, full - the last keeps whatever the sensor offers. */
    private static final double[][] ASPECTS = { {1, 1}, {4, 3}, {9, 16}, {0, 0} };
    private static final String[] ASPECT_LABELS = { "1:1", "4:3", "9:16", "全屏" };
    private static final String[] FLASH_LABELS = { "关闭", "自动", "开启", "常亮" };

    private static final String SP = "camera";
    private static final String PUBLIC_ALBUM = "DCIM/MTC";
    private static final String PUBLIC_VIDEO = "Movies/MTC";

    private HandlerThread worker;
    private Handler cameraHandler;
    private final Handler ui = new Handler(Looper.getMainLooper());

    private TextureView preview;
    /**
     * The one Surface that wraps the preview texture.
     *
     * <p>Creating a fresh Surface per request (which an earlier version did on every zoom,
     * focus tap and flash change) leaves the producer with Surfaces it never released, and
     * the symptom is a black viewfinder - the consumer stops getting buffers even though
     * the session still reports itself as configured.
     */
    private Surface previewSurface;
    private volatile int framesSeen;
    private boolean safeMode;
    private TextView diagnostics;
    private FrameLayout root;
    private ViewfinderOverlay overlay;
    private LinearLayout flashButton;
    private LinearLayout gridButton;
    private LinearLayout levelButton;
    private LinearLayout timerButton;
    private LinearLayout aspectButton;
    private LinearLayout zoomBar;
    private TextView[] modeButtons;
    private TextView recordTime;
    private View shutterInner;
    private ImageView thumb;

    private CameraDevice device;
    private CameraCaptureSession session;
    private ImageReader stills;
    private Size stillSize = new Size(3264, 2448);
    private Size previewSize = new Size(1440, 1080);
    private String cameraId;
    private Rect activeArray;
    private float maxDigitalZoom = 1f;

    /** Physical cameras behind the logical one, with the zoom each represents. */
    private final List<String> lensIds = new ArrayList<String>();
    private final List<Float> lensFactors = new ArrayList<Float>();
    private int lensIndex;

    private boolean busy;
    private int quality = 88;
    private int sizeChoice;
    private boolean fullRange;
    private boolean front;
    private int rotationOverride = -1;
    private int sensorOrientation = 90;

    private int flashMode;           // 0 off, 1 auto, 2 on, 3 torch
    private boolean flashAvailable;
    /** 0 = use the chosen size, 1..n = halve the still that many times after a failure. */
    private int stillDegrade;
    private boolean gridOn;
    private boolean levelOn;
    private int timerSeconds;        // 0, 2, 5, 10
    private int aspectIndex = 1;     // 4:3
    /** true keeps the whole frame visible (letterboxed); false fills the screen. */
    private boolean previewFit = true;
    private float requestZoom = 1f;  // digital zoom inside the current camera
    private int exposureCompensation;

    private MeteringRectangle[] focusAreas;
    private MeteringRectangle[] meteringAreas;

    private boolean videoMode;
    private MediaRecorder recorder;
    private ParcelFileDescriptor videoFile;
    private Uri videoUri;
    private Surface recorderSurface;
    private long recordStarted;
    private boolean recording;

    private boolean bursting;
    private int burstCount;

    /** Multi-frame state: the frames of a night or HDR shot as they arrive. */
    private Multi multi;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
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
        flashMode = sp.getInt("flash", 0);
        gridOn = sp.getBoolean("grid", false);
        levelOn = sp.getBoolean("level", false);
        timerSeconds = sp.getInt("timer", 0);
        aspectIndex = sp.getInt("aspect", 1);
        previewFit = sp.getBoolean("fit", true);
    }

    private void persist(String key, int value) {
        getSharedPreferences(SP, MODE_PRIVATE).edit().putInt(key, value).apply();
    }

    private void persist(String key, boolean value) {
        getSharedPreferences(SP, MODE_PRIVATE).edit().putBoolean(key, value).apply();
    }

    // ───────────────────────────── layout ─────────────────────────────

    private int dp(float value) {
        return Math.round(value * getResources().getDisplayMetrics().density);
    }

    private TextView icon(String text) {
        TextView view = new TextView(this);
        view.setText(text);
        view.setTextSize(17);
        view.setTextColor(Color.WHITE);
        view.setGravity(Gravity.CENTER);
        return view;
    }

    /**
     * A top-bar button: the glyph with its current value underneath.
     *
     * <p>The first version was glyphs alone, and nobody could tell what they did. A
     * two-character caption costs 10 dp and removes the guessing.
     */
    private LinearLayout topButton(String glyph, String caption) {
        LinearLayout holder = new LinearLayout(this);
        holder.setOrientation(LinearLayout.VERTICAL);
        holder.setGravity(Gravity.CENTER);
        TextView symbol = new TextView(this);
        symbol.setText(glyph);
        symbol.setTextSize(16);
        symbol.setTextColor(Color.WHITE);
        symbol.setGravity(Gravity.CENTER);
        symbol.setTag("symbol");
        holder.addView(symbol, new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT, dp(22)));
        TextView label = new TextView(this);
        label.setText(caption);
        label.setTextSize(9.5f);
        label.setTextColor(0xFFB9B9C2);
        label.setGravity(Gravity.CENTER);
        label.setTag("caption");
        holder.addView(label, new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT, dp(13)));
        return holder;
    }

    private void setCaption(LinearLayout holder, String caption) {
        TextView label = (TextView) holder.findViewWithTag("caption");
        if (label != null) {
            label.setText(caption);
        }
    }

    private void tintSymbol(LinearLayout holder, int color) {
        TextView symbol = (TextView) holder.findViewWithTag("symbol");
        if (symbol != null) {
            symbol.setTextColor(color);
        }
    }

    private void buildUi() {
        root = new FrameLayout(this);
        root.setBackgroundColor(Color.BLACK);

        preview = new TextureView(this);
        root.addView(preview, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        preview.setSurfaceTextureListener(new TextureView.SurfaceTextureListener() {
            @Override
            public void onSurfaceTextureAvailable(SurfaceTexture s, int w, int h) {
                try {
                    if (previewSurface != null) {
                        previewSurface.release();
                    }
                    previewSurface = new Surface(s);
                } catch (Exception ignored) {
                }
                openCamera();
            }

            @Override
            public void onSurfaceTextureSizeChanged(SurfaceTexture s, int w, int h) {
                configureTransform();
            }

            @Override
            public boolean onSurfaceTextureDestroyed(SurfaceTexture s) {
                if (previewSurface != null) {
                    previewSurface.release();
                    previewSurface = null;
                }
                return true;
            }

            @Override
            public void onSurfaceTextureUpdated(SurfaceTexture s) {
                framesSeen++;
            }
        });

        overlay = new ViewfinderOverlay(this);
        overlay.setListener(this);
        overlay.setGrid(gridOn);
        overlay.setLevel(levelOn);
        root.addView(overlay, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));

        final ScaleGestureDetector pinch = new ScaleGestureDetector(this,
                new ScaleGestureDetector.SimpleOnScaleGestureListener() {
                    @Override
                    public boolean onScale(ScaleGestureDetector detector) {
                        setZoom(requestZoom * detector.getScaleFactor());
                        return true;
                    }
                });
        preview.setOnTouchListener(new View.OnTouchListener() {
            @Override
            public boolean onTouch(View v, MotionEvent event) {
                pinch.onTouchEvent(event);
                return false;
            }
        });

        buildTopBar();
        buildZoomBar();
        buildModeStrip();
        buildBottomBar();

        recordTime = new TextView(this);
        recordTime.setTextColor(Color.WHITE);
        recordTime.setTextSize(13);
        recordTime.setPadding(dp(10), dp(4), dp(10), dp(4));
        recordTime.setBackgroundColor(0x99000000);
        recordTime.setVisibility(View.GONE);
        FrameLayout.LayoutParams recLp = new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT,
                Gravity.TOP | Gravity.CENTER_HORIZONTAL);
        recLp.setMargins(0, dp(64), 0, 0);
        root.addView(recordTime, recLp);

        setContentView(root);
        updateModeUi();
        updateTopLabels();
    }

    private void buildTopBar() {
        LinearLayout top = new LinearLayout(this);
        top.setOrientation(LinearLayout.HORIZONTAL);
        top.setGravity(Gravity.CENTER_VERTICAL);
        top.setPadding(dp(6), dp(6), dp(6), dp(6));
        root.addView(top, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, dp(58), Gravity.TOP));

        flashButton = topButton("⚡", "闪光 关");
        top.addView(flashButton, new LinearLayout.LayoutParams(dp(62), dp(48)));
        flashButton.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                if (!flashAvailable) {
                    toast("这台相机没有闪光灯");
                    return;
                }
                flashMode = (flashMode + 1) % 4;
                persist("flash", flashMode);
                updateTopLabels();
                applyTorch();
            }
        });

        gridButton = topButton("#", "网格 关");
        top.addView(gridButton, new LinearLayout.LayoutParams(dp(52), dp(48)));
        gridButton.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                gridOn = !gridOn;
                persist("grid", gridOn);
                overlay.setGrid(gridOn);
                updateTopLabels();
            }
        });

        levelButton = topButton("⌁", "水平 关");
        top.addView(levelButton, new LinearLayout.LayoutParams(dp(52), dp(48)));
        levelButton.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                levelOn = !levelOn;
                persist("level", levelOn);
                overlay.setLevel(levelOn);
                updateTopLabels();
            }
        });

        timerButton = topButton("⏱", "定时 关");
        top.addView(timerButton, new LinearLayout.LayoutParams(dp(52), dp(48)));
        timerButton.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                timerSeconds = timerSeconds == 0 ? 2 : timerSeconds == 2 ? 5
                        : timerSeconds == 5 ? 10 : 0;
                persist("timer", timerSeconds);
                updateTopLabels();
            }
        });

        aspectButton = topButton("▭", "4:3");
        top.addView(aspectButton, new LinearLayout.LayoutParams(dp(52), dp(48)));
        aspectButton.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                aspectIndex = (aspectIndex + 1) % ASPECTS.length;
                persist("aspect", aspectIndex);
                updateTopLabels();
                closeCamera();
                openCamera();
            }
        });

        LinearLayout more = topButton("⋯", "设置");
        top.addView(more, new LinearLayout.LayoutParams(0, dp(48), 1f));
        more.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                showSettings();
            }
        });
        // Long press for the numbers: the one thing that makes "it is black" debuggable.
        more.setOnLongClickListener(new View.OnLongClickListener() {
            @Override
            public boolean onLongClick(View v) {
                if (diagnostics != null && diagnostics.getVisibility() == View.VISIBLE) {
                    diagnostics.setVisibility(View.GONE);
                } else {
                    showDiagnostics();
                }
                return true;
            }
        });
    }

    private void updateTopLabels() {
        setCaption(flashButton, flashAvailable
                ? "闪光 " + (flashMode == 0 ? "关" : FLASH_LABELS[flashMode])
                : "闪光 无");
        setCaption(gridButton, gridOn ? "网格 开" : "网格 关");
        setCaption(levelButton, levelOn ? "水平 开" : "水平 关");
        setCaption(timerButton, timerSeconds == 0 ? "定时 关" : "定时 " + timerSeconds + "s");
        setCaption(aspectButton, ASPECT_LABELS[aspectIndex]);
        tintSymbol(flashButton, flashMode == 0 ? 0xFFB9B9C2 : 0xFFFFD166);
        tintSymbol(gridButton, gridOn ? 0xFFFFD166 : 0xFFB9B9C2);
        tintSymbol(levelButton, levelOn ? 0xFFFFD166 : 0xFFB9B9C2);
        tintSymbol(timerButton, timerSeconds == 0 ? 0xFFB9B9C2 : 0xFFFFD166);
        tintSymbol(aspectButton, aspectIndex == 3 ? 0xFFB9B9C2 : 0xFFFFD166);
    }
    /** The zoom bar: one entry per physical camera the device reports. */
    private void buildZoomBar() {
        HorizontalScrollView scroll = new HorizontalScrollView(this);
        scroll.setHorizontalScrollBarEnabled(false);
        zoomBar = new LinearLayout(this);
        zoomBar.setOrientation(LinearLayout.HORIZONTAL);
        zoomBar.setGravity(Gravity.CENTER);
        scroll.addView(zoomBar);
        FrameLayout.LayoutParams lp = new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT, dp(40),
                Gravity.BOTTOM | Gravity.CENTER_HORIZONTAL);
        lp.setMargins(0, 0, 0, dp(158));
        root.addView(scroll, lp);
    }

    private void buildModeStrip() {
        HorizontalScrollView scroll = new HorizontalScrollView(this);
        scroll.setHorizontalScrollBarEnabled(false);
        scroll.setFillViewport(true);
        LinearLayout strip = new LinearLayout(this);
        strip.setOrientation(LinearLayout.HORIZONTAL);
        strip.setGravity(Gravity.CENTER);
        scroll.addView(strip);
        FrameLayout.LayoutParams lp = new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, dp(44), Gravity.BOTTOM);
        lp.setMargins(0, 0, 0, dp(112));
        root.addView(scroll, lp);

        String[] modes = { "专业", "录像", "拍照", "人像", "夜景", "全景", "HDR" };
        modeButtons = new TextView[modes.length];
        for (int i = 0; i < modes.length; i++) {
            final String name = modes[i];
            TextView button = new TextView(this);
            button.setText(name);
            button.setTextSize(13);
            button.setGravity(Gravity.CENTER);
            modeButtons[i] = button;
            strip.addView(button, new LinearLayout.LayoutParams(dp(64),
                    ViewGroup.LayoutParams.MATCH_PARENT));
            button.setOnClickListener(new View.OnClickListener() {
                @Override
                public void onClick(View v) {
                    selectMode(name);
                }
            });
        }
    }

    private void selectMode(String name) {
        if (recording) {
            toast("先停止录制");
            return;
        }
        if ("拍照".equals(name)) {
            videoMode = false;
            updateModeUi();
            closeCamera();
            openCamera();
        } else if ("录像".equals(name)) {
            videoMode = true;
            updateModeUi();
            closeCamera();
            openCamera();
        } else if ("专业".equals(name)) {
            showProPanel();
        } else {
            toast(name + "：需要多帧合成，还没做");
        }
    }

    private void buildBottomBar() {
        LinearLayout bar = new LinearLayout(this);
        bar.setOrientation(LinearLayout.HORIZONTAL);
        bar.setGravity(Gravity.CENTER_VERTICAL);
        bar.setBackgroundColor(0xB3000000);
        root.addView(bar, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, dp(108), Gravity.BOTTOM));

        thumb = new ImageView(this);
        thumb.setScaleType(ImageView.ScaleType.CENTER_CROP);
        GradientDrawable thumbBg = new GradientDrawable();
        thumbBg.setColor(0x33FFFFFF);
        thumbBg.setCornerRadius(dp(6));
        thumb.setBackground(thumbBg);
        LinearLayout.LayoutParams thumbLp = new LinearLayout.LayoutParams(dp(52), dp(52));
        thumbLp.setMargins(dp(16), 0, 0, 0);
        bar.addView(thumb, thumbLp);
        thumb.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                startActivity(new android.content.Intent(CameraActivity.this,
                        GalleryActivity.class));
            }
        });

        FrameLayout shutterHolder = new FrameLayout(this);
        bar.addView(shutterHolder, new LinearLayout.LayoutParams(0,
                ViewGroup.LayoutParams.MATCH_PARENT, 1f));

        View ring = new View(this);
        GradientDrawable ringDraw = new GradientDrawable();
        ringDraw.setShape(GradientDrawable.OVAL);
        ringDraw.setColor(Color.TRANSPARENT);
        ringDraw.setStroke(dp(4), Color.WHITE);
        ring.setBackground(ringDraw);
        shutterHolder.addView(ring, new FrameLayout.LayoutParams(dp(72), dp(72), Gravity.CENTER));

        shutterInner = new View(this);
        shutterHolder.addView(shutterInner, new FrameLayout.LayoutParams(dp(56), dp(56),
                Gravity.CENTER));

        shutterHolder.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                if (videoMode) {
                    toggleRecording();
                } else {
                    shutterPressed();
                }
            }
        });
        shutterHolder.setOnLongClickListener(new View.OnLongClickListener() {
            @Override
            public boolean onLongClick(View v) {
                if (!videoMode) {
                    startBurst();
                }
                return true;
            }
        });
        shutterHolder.setOnTouchListener(new View.OnTouchListener() {
            @Override
            public boolean onTouch(View v, MotionEvent event) {
                if (event.getAction() == MotionEvent.ACTION_UP && bursting) {
                    stopBurst();
                }
                return false;
            }
        });

        TextView flip = icon("⇄");
        flip.setTextSize(22);
        LinearLayout.LayoutParams flipLp = new LinearLayout.LayoutParams(dp(56), dp(56));
        flipLp.setMargins(0, 0, dp(12), 0);
        bar.addView(flip, flipLp);
        flip.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                if (recording) {
                    toast("录制中不能切换");
                    return;
                }
                front = !front;
                lensIds.clear();
                lensFactors.clear();
                closeCamera();
                openCamera();
            }
        });
    }

    private void updateModeUi() {
        if (modeButtons == null) {
            return;
        }
        String active = videoMode ? "录像" : "拍照";
        for (int i = 0; i < modeButtons.length; i++) {
            boolean on = active.equals(modeButtons[i].getText().toString());
            modeButtons[i].setTextColor(on ? 0xFFFFD166 : Color.WHITE);
            modeButtons[i].setTextSize(on ? 14 : 13);
        }
        GradientDrawable dot = new GradientDrawable();
        dot.setShape(GradientDrawable.OVAL);
        dot.setColor(recording ? Color.parseColor("#FF3B30") : Color.WHITE);
        shutterInner.setBackground(dot);
    }

    // ───────────────────────────── viewfinder gestures ─────────────────────────────

    @Override
    public void onFocus(float x, float y) {
        if (device == null || preview.getWidth() == 0 || preview.getSurfaceTexture() == null) {
            return;
        }
        Rect box = cameraRegionOf(x, y);
        focusAreas = new MeteringRectangle[]{
                new MeteringRectangle(box, MeteringRectangle.METERING_WEIGHT_MAX)};
        meteringAreas = focusAreas;
        try {
            CaptureRequest.Builder trigger =
                    device.createCaptureRequest(CameraDevice.TEMPLATE_PREVIEW);
            if (previewSurface == null) {
                return;
            }
            trigger.addTarget(previewSurface);
            trigger.set(CaptureRequest.CONTROL_AF_TRIGGER, CaptureRequest.CONTROL_AF_TRIGGER_START);
            trigger.set(CaptureRequest.CONTROL_AF_REGIONS, focusAreas);
            trigger.set(CaptureRequest.CONTROL_AE_REGIONS, meteringAreas);
            session.capture(trigger.build(), null, cameraHandler);
        } catch (Exception ignored) {
        }
        applyRepeating();
    }

    /** Map a point on the view onto the sensor's active array. */
    private Rect cameraRegionOf(float x, float y) {
        if (activeArray == null) {
            return new Rect(0, 0, 1, 1);
        }
        float vw = Math.max(1, preview.getWidth());
        float vh = Math.max(1, preview.getHeight());
        int span = (int) (Math.min(activeArray.width(), activeArray.height()) * 0.18f);
        int cx = activeArray.left + (int) (x / vw * activeArray.width());
        int cy = activeArray.top + (int) (y / vh * activeArray.height());
        Rect rect = new Rect(cx - span / 2, cy - span / 2, cx + span / 2, cy + span / 2);
        rect.offset(Math.max(0, activeArray.left - rect.left), 0);
        rect.offset(0, Math.max(0, activeArray.top - rect.top));
        rect.offset(Math.min(0, activeArray.right - rect.right), 0);
        rect.offset(0, Math.min(0, activeArray.bottom - rect.bottom));
        return rect;
    }

    @Override
    public void onExposure(float ev) {
        exposureCompensation = Math.round(ev);
        applyRepeating();
    }

    private void setZoom(float zoom) {
        requestZoom = Math.max(1f, Math.min(maxDigitalZoom, zoom));
        applyRepeating();
    }

    private void updateZoomLabels() {
        if (zoomBar == null) {
            return;
        }
        for (int i = 0; i < zoomBar.getChildCount(); i++) {
            TextView label = (TextView) zoomBar.getChildAt(i);
            label.setTextColor(i == lensIndex ? 0xFFFFD166 : Color.WHITE);
        }
    }

    /** One button per physical camera; tapping it switches lens. */
    private void fillZoomBar(CameraCharacteristics chars, CameraManager manager) {
        lensIds.clear();
        lensFactors.clear();
        try {
            java.util.Set<String> physical = chars.getPhysicalCameraIds();
            List<String> ids = new ArrayList<String>();
            if (physical != null && physical.size() > 1) {
                ids.addAll(physical);
                Collections.sort(ids);
            } else if (cameraId != null) {
                ids.add(cameraId);
            }
            float mainFocal = 0f;
            for (int i = 0; i < ids.size(); i++) {
                CameraCharacteristics c = manager.getCameraCharacteristics(ids.get(0));
                float[] focals = c.get(CameraCharacteristics.LENS_INFO_AVAILABLE_FOCAL_LENGTHS);
                mainFocal = focals == null || focals.length == 0 ? 1f : focals[0];
            }
            for (int i = 0; i < ids.size(); i++) {
                CameraCharacteristics c = manager.getCameraCharacteristics(ids.get(i));
                float[] focals = c.get(CameraCharacteristics.LENS_INFO_AVAILABLE_FOCAL_LENGTHS);
                float focal = focals == null || focals.length == 0 ? mainFocal : focals[0];
                lensIds.add(ids.get(i));
                lensFactors.add(mainFocal <= 0 ? 1f : Math.max(0.5f, focal / mainFocal));
            }
        } catch (Exception e) {
            lensIds.clear();
            lensFactors.clear();
        }
        if (lensIds.isEmpty() && cameraId != null) {
            lensIds.add(cameraId);
            lensFactors.add(1f);
        }
        lensIndex = 0;
        if (zoomBar != null) {
            zoomBar.removeAllViews();
            for (int i = 0; i < lensFactors.size(); i++) {
                final int index = i;
                TextView label = new TextView(this);
                float factor = lensFactors.get(i);
                label.setText(factor >= 1f
                        ? String.format(Locale.US, "%.1fx", factor)
                        : String.format(Locale.US, "0.%dx", Math.round(factor * 10)));
                label.setTextSize(13);
                label.setPadding(dp(12), dp(6), dp(12), dp(6));
                zoomBar.addView(label);
                label.setOnClickListener(new View.OnClickListener() {
                    @Override
                    public void onClick(View v) {
                        switchLens(index);
                    }
                });
            }
            updateZoomLabels();
        }
    }

    private void switchLens(int index) {
        if (index < 0 || index >= lensIds.size() || index == lensIndex) {
            return;
        }
        lensIndex = index;
        requestZoom = 1f;
        updateZoomLabels();
        closeCamera();
        openCamera();
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
            if (lensIds.isEmpty()) {
                cameraId = pickCamera(manager, front);
                if (cameraId == null) {
                    toast("找不到相机");
                    return;
                }
            } else {
                cameraId = lensIds.get(Math.min(lensIndex, lensIds.size() - 1));
            }
            CameraCharacteristics chars = manager.getCameraCharacteristics(cameraId);
            activeArray = chars.get(CameraCharacteristics.SENSOR_INFO_ACTIVE_ARRAY_SIZE);
            Float maxZoom = chars.get(CameraCharacteristics.SCALER_AVAILABLE_MAX_DIGITAL_ZOOM);
            maxDigitalZoom = maxZoom == null ? 1f : Math.max(1f, maxZoom);
            if (lensIds.isEmpty()) {
                fillZoomBar(chars, manager);
            }

            StreamConfigurationMap map =
                    chars.get(CameraCharacteristics.SCALER_STREAM_CONFIGURATION_MAP);
            stillSize = pickStillSize(map);
            if (safeMode) {
                // The smallest thing a camera is guaranteed to support.
                stillSize = new Size(640, 480);
            }
            for (int i = 0; i < stillDegrade; i++) {
                stillSize = new Size(Math.max(320, stillSize.getWidth() / 2),
                        Math.max(240, stillSize.getHeight() / 2));
            }
            previewSize = pickPreviewSize(map, stillSize);
            if (safeMode) {
                previewSize = new Size(640, 480);
            }
            Boolean hasFlash = chars.get(CameraCharacteristics.FLASH_INFO_AVAILABLE);
            flashAvailable = hasFlash != null && hasFlash.booleanValue();
            if (!flashAvailable) {
                flashMode = 0;
            }
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
            CameraCharacteristics chars = manager.getCameraCharacteristics(ids[i]);
            Integer facing = chars.get(CameraCharacteristics.LENS_FACING);
            if (facing != null && facing.intValue() == wanted) {
                return ids[i];
            }
            if (fallback == null) {
                fallback = ids[i];
            }
        }
        return fallback;
    }

    private long maxPixels() {
        int choice = Math.max(0, Math.min(SIZE_CHOICES.length - 1, sizeChoice));
        return SIZE_CHOICES[choice][0];
    }

    /** The chosen frame shape wins; otherwise 4:3, then 16:9, then largest. */
    private Size pickStillSize(StreamConfigurationMap map) {
        Size[] sizes = map == null ? null
                : map.getOutputSizes(android.graphics.ImageFormat.YUV_420_888);
        if (sizes == null || sizes.length == 0) {
            return new Size(1920, 1080);
        }
        if (aspectIndex < 3) {
            double want = ASPECTS[aspectIndex][0] / ASPECTS[aspectIndex][1];
            Size framed = pickByAspect(sizes, want);
            if (framed != null) {
                return framed;
            }
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

    /** Rotate by the same angle the picture gets, and scale uniformly - never stretch. */
    private void configureTransform() {
        if (preview.getWidth() == 0 || preview.getHeight() == 0) {
            return;
        }
        int deg = stillRotation();
        if (safeMode) {
            deg = 0;
        }
        float vw = preview.getWidth();
        float vh = preview.getHeight();
        boolean swapped = (deg % 180) != 0;
        float bw = swapped ? previewSize.getHeight() : previewSize.getWidth();
        float bh = swapped ? previewSize.getWidth() : previewSize.getHeight();
        // Fit keeps the whole frame on screen and letterboxes the rest; fill crops
        // instead. Neither stretches: the scale is one number for both axes either way.
        float scale = previewFit ? Math.min(vw / bw, vh / bh) : Math.max(vw / bw, vh / bh);
        android.graphics.Matrix matrix = new android.graphics.Matrix();
        matrix.postRotate(deg);
        matrix.postScale(scale, scale);
        matrix.postTranslate((vw - bw * scale) / 2f, (vh - bh * scale) / 2f);
        preview.setTransform(matrix);
    }

    private CaptureRequest.Builder baseRequest(int template, Surface target)
            throws CameraAccessException {
        CaptureRequest.Builder builder = device.createCaptureRequest(template);
        if (target != null) {
            builder.addTarget(target);
        }
        builder.set(CaptureRequest.CONTROL_MODE, CaptureRequest.CONTROL_MODE_AUTO);
        builder.set(CaptureRequest.CONTROL_AF_MODE,
                CaptureRequest.CONTROL_AF_MODE_CONTINUOUS_PICTURE);
        // Flash belongs to the still capture, not to the preview: asking a preview
        // request for ALWAYS_FLASH is rejected by the camera, the session never
        // configures, and the viewfinder stays black. That is exactly what happened.
        boolean still = template == CameraDevice.TEMPLATE_STILL_CAPTURE;
        builder.set(CaptureRequest.CONTROL_AE_MODE,
                still && flashAvailable ? aeMode() : CaptureRequest.CONTROL_AE_MODE_ON);
        if (exposureCompensation != 0) {
            builder.set(CaptureRequest.CONTROL_AE_EXPOSURE_COMPENSATION, exposureCompensation);
        }
        if (focusAreas != null) {
            builder.set(CaptureRequest.CONTROL_AF_REGIONS, focusAreas);
            builder.set(CaptureRequest.CONTROL_AE_REGIONS, meteringAreas);
        }
        if (activeArray != null && requestZoom > 1.01f) {
            int w = activeArray.width();
            int h = activeArray.height();
            int dw = (int) (w / requestZoom);
            int dh = (int) (h / requestZoom);
            int left = activeArray.left + (w - dw) / 2;
            int top = activeArray.top + (h - dh) / 2;
            builder.set(CaptureRequest.SCALER_CROP_REGION, new Rect(left, top, left + dw, top + dh));
        }
        return builder;
    }

    private int aeMode() {
        switch (flashMode) {
            case 2:
                return CaptureRequest.CONTROL_AE_MODE_ON_ALWAYS_FLASH;
            case 1:
                return CaptureRequest.CONTROL_AE_MODE_ON_AUTO_FLASH;
            default:
                return CaptureRequest.CONTROL_AE_MODE_ON;
        }
    }

    private void applyTorch() {
        try {
            CameraManager manager = (CameraManager) getSystemService(Context.CAMERA_SERVICE);
            manager.setTorchMode(cameraId, flashMode == 3);
        } catch (Exception ignored) {
        }
    }

    private void applyRepeating() {
        if (session == null || device == null || previewSurface == null) {
            return;
        }
        try {
            CaptureRequest.Builder builder =
                    baseRequest(CameraDevice.TEMPLATE_PREVIEW, previewSurface);
            session.setRepeatingRequest(builder.build(), null, cameraHandler);
        } catch (Exception ignored) {
        }
    }

    private void startPreview() {
        if (device == null || preview.getSurfaceTexture() == null) {
            return;
        }
        try {
            SurfaceTexture texture = preview.getSurfaceTexture();
            texture.setDefaultBufferSize(previewSize.getWidth(), previewSize.getHeight());
            Surface surface = new Surface(texture);
            CaptureRequest.Builder builder = baseRequest(CameraDevice.TEMPLATE_PREVIEW, surface);

            List<Surface> targets = new ArrayList<Surface>();
            targets.add(surface);
            if (!videoMode && !safeMode && stills != null) {
                targets.add(stills.getSurface());
            }
            final CaptureRequest request = builder.build();
            device.createCaptureSession(targets, new CameraCaptureSession.StateCallback() {
                @Override
                public void onConfigured(CameraCaptureSession configured) {
                    session = configured;
                    try {
                        session.setRepeatingRequest(request, null, cameraHandler);
                        framesSeen = 0;
                        startWatchdog();
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
                    // A camera can refuse a session outright - too large a still, or a
                    // combination a container does not implement. Rather than leave a
                    // black screen, step the still size down and try again.
                    if (stillDegrade < 3) {
                        stillDegrade++;
                        session = null;
                        closeCamera();
                        openCamera();
                        toast("这台相机吃不消这个尺寸，降到第 " + stillDegrade + " 档重试");
                    } else {
                        toast("预览配置失败：相机不接受这个组合");
                    }
                }
            }, cameraHandler);
        } catch (CameraAccessException e) {
            toast("预览出错：" + e.getMessage());
        }
    }


    /**
     * If no frame has been drawn a moment after the session configured, something is
     * wrong in a way the session callback cannot see, so say so and try the safest
     * configuration there is.
     *
     * <p>"Black screen" is the hardest bug report to act on - the session reports success,
     * no exception is thrown, and the picture simply never arrives. A frame counter is the
     * only honest signal, and it turns the report into either a working preview or a line
     * of text with the numbers in it.
     */
    private void startWatchdog() {
        ui.postDelayed(new Runnable() {
            @Override
            public void run() {
                if (framesSeen > 0 || device == null) {
                    return;
                }
                if (!safeMode) {
                    safeMode = true;
                    toast("取景没有画面，切到安全模式重试");
                    closeCamera();
                    openCamera();
                } else {
                    showDiagnostics();
                    toast("安全模式也没有画面，屏幕上那一行是原因");
                }
            }
        }, 2500);
    }

    /** A line of numbers over the viewfinder, for when a picture is not arriving. */
    private void showDiagnostics() {
        if (diagnostics == null) {
            diagnostics = new TextView(this);
            diagnostics.setTextSize(10.5f);
            diagnostics.setTextColor(0xFFFFD166);
            diagnostics.setBackgroundColor(0xCC000000);
            diagnostics.setPadding(dp(8), dp(4), dp(8), dp(4));
            FrameLayout.LayoutParams lp = new FrameLayout.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT,
                    Gravity.BOTTOM);
            lp.setMargins(0, 0, 0, dp(160));
            root.addView(diagnostics, lp);
        }
        diagnostics.setVisibility(View.VISIBLE);
        diagnostics.setText(String.format(Locale.US,
                "设备 %s · 会话 %s · 面 %s · 帧 %d · 预览 %dx%d · 拍照 %dx%d · 传感器 %d° → %d° · 安全 %s",
                device != null ? "有" : "无",
                session != null ? "有" : "无",
                previewSurface != null ? "有" : "无",
                framesSeen,
                previewSize.getWidth(), previewSize.getHeight(),
                stillSize.getWidth(), stillSize.getHeight(),
                sensorOrientation, stillRotation(),
                safeMode ? "开" : "关"));
    }    private void closeCamera() {
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

    // ───────────────────────────── shutter ─────────────────────────────

    private void shutterPressed() {
        if (timerSeconds > 0) {
            countdown(timerSeconds);
        } else {
            capture();
        }
    }

    private void countdown(final int remaining) {
        if (remaining <= 0) {
            capture();
            return;
        }
        toast(String.valueOf(remaining));
        ui.postDelayed(new Runnable() {
            @Override
            public void run() {
                countdown(remaining - 1);
            }
        }, 1000);
    }

    private void startBurst() {
        bursting = true;
        burstCount = 0;
        burstStep();
    }

    private void burstStep() {
        if (!bursting) {
            return;
        }
        if (burstCount >= 20) {
            stopBurst();
            return;
        }
        if (!busy) {
            burstCount++;
            capture();
        }
        ui.postDelayed(new Runnable() {
            @Override
            public void run() {
                burstStep();
            }
        }, 420);
    }

    private void stopBurst() {
        if (!bursting) {
            return;
        }
        bursting = false;
        toast("连拍 " + burstCount + " 张");
    }

    private void capture() {
        if (device == null || session == null || busy) {
            return;
        }
        busy = true;
        try {
            CaptureRequest.Builder builder =
                    baseRequest(CameraDevice.TEMPLATE_STILL_CAPTURE, stills.getSurface());
            session.capture(builder.build(), null, cameraHandler);
        } catch (CameraAccessException e) {
            busy = false;
            toast("拍摄失败：" + e.getMessage());
        }
    }

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

            long t0 = System.currentTimeMillis();
            final byte[] encoded = Mtc.encodeYuv(y, u, v,
                    image.getWidth(), image.getHeight(),
                    planes[0].getRowStride(), planes[1].getRowStride(), planes[1].getPixelStride(),
                    quality, fullRange, stillRotation(), front);
            final long ms = System.currentTimeMillis() - t0;
            final Size size = stillSize;
            image.close();
            image = null;
            if (encoded == null) {
                toast("编码失败");
                return;
            }
            final String name = "IMG_" + new SimpleDateFormat("yyyyMMdd_HHmmss_SSS",
                    Locale.US).format(new Date()) + ".mtc";
            savePicture(name, encoded);
            ui.post(new Runnable() {
                @Override
                public void run() {
                    if (!bursting) {
                        toast(Store.human(encoded.length) + " · " + ms + " ms · "
                                + size.getWidth() + "×" + size.getHeight());
                    }
                    loadThumb(name);
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


    /** One multi-frame shot: how many frames, at what exposures, merged which way. */
    private static final class Multi {
        final int want;
        final float[] exposures;
        final int mode;          // 0 night, 1 hdr
        final java.util.ArrayList<byte[][]> planes = new java.util.ArrayList<byte[][]>();
        final java.util.ArrayList<int[]> geometry = new java.util.ArrayList<int[]>();
        int w;
        int h;

        Multi(int want, float[] exposures, int mode) {
            this.want = want;
            this.exposures = exposures;
            this.mode = mode;
        }
    }

    /**
     * Take a burst and merge it: the night and HDR modes.
     *
     * <p>Runs at 1080p rather than full resolution on purpose. Six frames of 8 MP RGB is
     * 144 MB before the merge even starts, which is where a phone starts killing apps;
     * 1080p keeps the whole thing inside a few tens of megabytes, and the point of these
     * modes is noise and dynamic range, not pixel count.
     */
    private void shootMulti(final int count, final float[] exposures, final int mode) {
        if (device == null || multi != null) {
            return;
        }
        final Size burst = burstSize();
        if (stills != null) {
            stills.close();
            stills = null;
        }
        multi = new Multi(count, exposures, mode);
        multi.w = burst.getWidth();
        multi.h = burst.getHeight();
        final ImageReader reader = ImageReader.newInstance(burst.getWidth(), burst.getHeight(),
                android.graphics.ImageFormat.YUV_420_888, count);
        final ImageReader[] holder = { reader };
        reader.setOnImageAvailableListener(new ImageReader.OnImageAvailableListener() {
            @Override
            public void onImageAvailable(ImageReader source) {
                onMultiFrame(source);
            }
        }, cameraHandler);

        try {
            final Surface recordPreview = previewSurface;
            List<Surface> targets = new ArrayList<Surface>();
            targets.add(recordPreview);
            targets.add(reader.getSurface());
            closeSession();
            device.createCaptureSession(targets, new CameraCaptureSession.StateCallback() {
                @Override
                public void onConfigured(CameraCaptureSession configured) {
                    session = configured;
                    try {
                        CaptureRequest.Builder previewBuilder =
                                baseRequest(CameraDevice.TEMPLATE_PREVIEW, recordPreview);
                        session.setRepeatingRequest(previewBuilder.build(), null, cameraHandler);
                        CaptureRequest.Builder still;
                        for (int i = 0; i < count; i++) {
                            still = baseRequest(CameraDevice.TEMPLATE_STILL_CAPTURE,
                                    reader.getSurface());
                            if (exposures != null && i < exposures.length) {
                                still.set(CaptureRequest.CONTROL_AE_EXPOSURE_COMPENSATION,
                                        Math.round(exposures[i]));
                            }
                            session.capture(still.build(), null, cameraHandler);
                        }
                        toast(mode == 1 ? "拍 3 帧合成 HDR…" : "拍 " + count + " 帧降噪…");
                    } catch (Exception e) {
                        finishMulti();
                        toast("多帧拍摄失败：" + e.getMessage());
                    }
                }

                @Override
                public void onConfigureFailed(CameraCaptureSession failed) {
                    finishMulti();
                    toast("多帧会话配置失败");
                }
            }, cameraHandler);
        } catch (Exception e) {
            finishMulti();
            toast("多帧打不开：" + e.getMessage());
        }
    }

    /** The size a burst runs at: the still size when that is already small enough. */
    private Size burstSize() {
        long cap = 1920L * 1080L;
        long pixels = (long) stillSize.getWidth() * stillSize.getHeight();
        if (pixels <= cap) {
            return stillSize;
        }
        double scale = Math.sqrt((double) cap / pixels);
        int w = Math.max(320, (int) (stillSize.getWidth() * scale) / 2 * 2);
        int h = Math.max(240, (int) (stillSize.getHeight() * scale) / 2 * 2);
        return new Size(w, h);
    }

    private void onMultiFrame(ImageReader reader) {
        Image image = null;
        try {
            image = reader.acquireNextImage();
            if (image == null || multi == null) {
                return;
            }
            Image.Plane[] planes = image.getPlanes();
            byte[][] copy = new byte[3][];
            for (int i = 0; i < 3; i++) {
                ByteBuffer buffer = planes[i].getBuffer();
                copy[i] = new byte[buffer.remaining()];
                buffer.get(copy[i]);
            }
            multi.planes.add(copy);
            multi.geometry.add(new int[]{
                    planes[0].getRowStride(), planes[1].getRowStride(), planes[1].getPixelStride()});
            if (multi.planes.size() >= multi.want) {
                finishMulti();
            }
        } catch (Exception ignored) {
        } finally {
            if (image != null) {
                image.close();
            }
        }
    }

    /** Merge what arrived, encode it, save it. */
    private void finishMulti() {
        final Multi shot = multi;
        multi = null;
        if (shot == null) {
            return;
        }
        try {
            if (session != null) {
                session.stopRepeating();
            }
        } catch (Exception ignored) {
        }
        try {
            final int n = shot.planes.size();
            if (n == 0) {
                toast("一帧都没拍到");
                closeCamera();
                openCamera();
                return;
            }
            final int w = shot.w;
            final int h = shot.h;
            byte[] flat = new byte[n * w * h * 3];
            for (int i = 0; i < n; i++) {
                byte[][] planes = shot.planes.get(i);
                int[] strides = shot.geometry.get(i);
                yuvToRgb(planes, strides, w, h, flat, i * w * h * 3);
            }
            byte[] evs = new byte[n * 4];
            for (int i = 0; i < n; i++) {
                float ev = shot.exposures == null || i >= shot.exposures.length ? 0f
                        : shot.exposures[i];
                int bits = Float.floatToIntBits(ev);
                evs[i * 4] = (byte) (bits & 0xFF);
                evs[i * 4 + 1] = (byte) ((bits >> 8) & 0xFF);
                evs[i * 4 + 2] = (byte) ((bits >> 16) & 0xFF);
                evs[i * 4 + 3] = (byte) ((bits >> 24) & 0xFF);
            }
            long t0 = System.currentTimeMillis();
            byte[] merged = Mtc.merge(flat, n, w, h, 3, shot.mode, evs);
            long ms = System.currentTimeMillis() - t0;
            if (merged == null) {
                toast("合成失败");
                closeCamera();
                openCamera();
                return;
            }
            byte[] rgba = new byte[w * h * 4];
            for (int i = 0, o = 0, q = 0; i < w * h; i++, o += 3, q += 4) {
                rgba[q] = merged[o];
                rgba[q + 1] = merged[o + 1];
                rgba[q + 2] = merged[o + 2];
                rgba[q + 3] = (byte) 255;
            }
            byte[] encoded = Mtc.encodeRgba(rgba, w, h, quality);
            if (encoded == null) {
                toast("编码失败");
                return;
            }
            final String name = ((shot.mode == 1 ? "HDR_" : "NIGHT_")
                    + new SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(new Date())
                    + ".mtc");
            savePicture(name, encoded);
            loadThumb(name);
            toast((shot.mode == 1 ? "HDR " : "夜景 ") + n + " 帧 → "
                    + Store.human(encoded.length) + " · 合成 " + ms + " ms");
        } catch (Exception e) {
            toast("合成出错：" + e.getMessage());
        }
        closeCamera();
        openCamera();
    }

    /**
     * YUV_420_888 to packed RGB, honouring the strides the camera chose.
     *
     * <p>Limited range (16-235) versus full range is the difference between a washed out
     * picture and a correct one, so the same flag the encoder uses decides it here too.
     */
    private void yuvToRgb(byte[][] planes, int[] strides, int w, int h, byte[] out, int offset) {
        int yStride = strides[0];
        int uStride = strides[1];
        int uvPixel = strides[2];
        for (int y = 0; y < h; y++) {
            int yRow = y * yStride;
            int uvRow = (y / 2) * uStride;
            for (int x = 0; x < w; x++) {
                int yv = planes[0][yRow + x] & 0xFF;
                int uvAt = uvRow + (x / 2) * uvPixel;
                int u = (planes[1][uvAt] & 0xFF) - 128;
                int v = (planes[2][uvAt] & 0xFF) - 128;
                int r;
                int g;
                int b;
                if (fullRange) {
                    int c = yv;
                    r = c + ((359 * v) >> 8);
                    g = c - ((88 * u + 183 * v) >> 8);
                    b = c + ((454 * u) >> 8);
                } else {
                    int c = Math.max(0, yv - 16) * 255 / 219;
                    r = c + ((359 * v) >> 8);
                    g = c - ((88 * u + 183 * v) >> 8);
                    b = c + ((454 * u) >> 8);
                }
                int at = offset + (y * w + x) * 3;
                out[at] = (byte) Math.max(0, Math.min(255, r));
                out[at + 1] = (byte) Math.max(0, Math.min(255, g));
                out[at + 2] = (byte) Math.max(0, Math.min(255, b));
            }
        }
    }    private void loadThumb(String name) {
        try {
            File file = new File(Store.dir(this), name);
            if (!file.exists()) {
                return;
            }
            if (!Store.hasThumbnail(file)) {
                Store.writeThumbnail(file, 320);
            }
            if (Store.hasThumbnail(file)) {
                android.graphics.Bitmap bmp = BitmapFactory.decodeFile(
                        Store.thumbFile(file).getAbsolutePath());
                if (bmp != null) {
                    thumb.setImageBitmap(bmp);
                }
            }
        } catch (Exception ignored) {
        }
    }

    private String savePicture(String name, byte[] bytes) {
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
            }
        }
        return new File(Store.dir(this), name).getAbsolutePath();
    }

    private void writePrivate(String name, byte[] bytes) {
        try {
            FileOutputStream fos = new FileOutputStream(new File(Store.dir(this), name));
            try {
                fos.write(bytes);
            } finally {
                fos.close();
            }
        } catch (Exception ignored) {
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
            rec.setOrientationHint(stillRotation());
            rec.setOutputFile(videoFile.getFileDescriptor());
            rec.prepare();
            recorder = rec;
            recorderSurface = rec.getSurface();

            final Surface recordPreview = previewSurface;
            List<Surface> targets = new ArrayList<Surface>();
            targets.add(recordPreview);
            targets.add(recorderSurface);
            closeSession();
            device.createCaptureSession(targets, new CameraCaptureSession.StateCallback() {
                @Override
                public void onConfigured(CameraCaptureSession configured) {
                    session = configured;
                    try {
                        CaptureRequest.Builder builder =
                                baseRequest(CameraDevice.TEMPLATE_RECORD, recordPreview);
                        builder.addTarget(recorderSurface);
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

    // ───────────────────────────── panels ─────────────────────────────

    private void showProPanel() {
        String[] items = {
                "ISO 感光度：自动",
                "快门 S：自动",
                "白平衡 WB：自动",
                "手动对焦 MF：关",
                "还没做：这三项目前只有自动",
        };
        new AlertDialog.Builder(this)
                .setTitle("专业模式")
                .setItems(items, null)
                .setPositiveButton("知道了", null)
                .show();
    }

    private void showSettings() {
        LinearLayout sheet = new LinearLayout(this);
        sheet.setOrientation(LinearLayout.VERTICAL);
        sheet.setPadding(dp(20), dp(8), dp(20), dp(8));

        final TextView resolution = row(sheet);
        final TextView qualityRow = row(sheet);
        final TextView rotation = row(sheet);
        final TextView range = row(sheet);
        final TextView fitRow = row(sheet);
        final TextView diagnostics = new TextView(this);
        diagnostics.setTextSize(11.5f);
        diagnostics.setTextColor(0xFF666670);
        diagnostics.setPadding(0, dp(12), 0, 0);
        sheet.addView(diagnostics);

        final Runnable refresh = new Runnable() {
            @Override
            public void run() {
                resolution.setText("分辨率：" + sizeLabel());
                qualityRow.setText("质量：q" + quality);
                rotation.setText(rotationOverride < 0
                        ? "方向：自动（传感器 " + sensorOrientation + "°）"
                        : "方向：固定 " + rotationOverride + "°");
                range.setText(fullRange ? "色彩范围：全 (0-255)" : "色彩范围：有限 (16-235)");
                fitRow.setText(previewFit ? "取景：完整（不裁切）" : "取景：填满屏幕（会裁切边缘）");
                diagnostics.setText(String.format(Locale.US,
                        "拍照 %d×%d · 预览 %d×%d · 传感器 %d° 转 %d°\n闪光 %s · 画幅 %s · 定时 %d s\n照片 → %s，视频 → %s",
                        stillSize.getWidth(), stillSize.getHeight(),
                        previewSize.getWidth(), previewSize.getHeight(),
                        sensorOrientation, stillRotation(), FLASH_LABELS[flashMode],
                        ASPECT_LABELS[aspectIndex], timerSeconds, PUBLIC_ALBUM, PUBLIC_VIDEO));
            }
        };
        refresh.run();

        resolution.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                sizeChoice = (sizeChoice + 1) % SIZE_CHOICES.length;
                persist("size", sizeChoice);
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
                persist("quality", quality);
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
                persist("rotation", rotationOverride);
                refresh.run();
                closeCamera();
                openCamera();
            }
        });
        range.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                fullRange = !fullRange;
                persist("fullRange", fullRange);
                refresh.run();
            }
        });

        fitRow.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                previewFit = !previewFit;
                persist("fit", previewFit);
                refresh.run();
                configureTransform();
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

    private TextView row(LinearLayout parent) {
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

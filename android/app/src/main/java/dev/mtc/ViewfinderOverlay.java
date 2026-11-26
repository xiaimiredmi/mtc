package dev.mtc;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.RectF;
import android.hardware.Sensor;
import android.hardware.SensorEvent;
import android.hardware.SensorEventListener;
import android.hardware.SensorManager;
import android.view.MotionEvent;
import android.view.View;

/**
 * What is drawn over the preview: the rule-of-thirds grid, the spirit level, the focus
 * box with its exposure slider.
 *
 * <p>Kept as a plain {@link View} rather than a pile of child views so it costs one
 * canvas and no layout work per frame - a viewfinder cannot afford jank.
 */
public class ViewfinderOverlay extends View implements SensorEventListener {
    /** Called when the user picks a focus point, in view coordinates. */
    public interface Listener {
        void onFocus(float x, float y);

        void onExposure(float ev);
    }

    private final Paint line = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Paint box = new Paint(Paint.ANTI_ALIAS_FLAG);
    private final Paint shade = new Paint();

    private boolean grid;
    private boolean level;
    private float tilt;          // degrees away from level, -1..1 of the bar
    private float roll;          // degrees of rotation around the view axis
    private boolean haveRoll;

    private float focusX = -1;
    private float focusY = -1;
    private float ev;            // -3..3
    private boolean showEv;

    private final SensorManager sensors;
    private final Sensor gravity;

    private Listener listener;
    /** Faces as boxes already in view coordinates, so this class stays dumb. */
    private final java.util.ArrayList<RectF> faces = new java.util.ArrayList<RectF>();

    public ViewfinderOverlay(Context context) {
        super(context);
        line.setStrokeWidth(dp(1f));
        line.setColor(0x66FFFFFF);
        box.setStyle(Paint.Style.STROKE);
        box.setStrokeWidth(dp(1.5f));
        box.setColor(0xCCFFD166);
        shade.setColor(0x66202020);
        sensors = (SensorManager) context.getSystemService(Context.SENSOR_SERVICE);
        gravity = sensors == null ? null : sensors.getDefaultSensor(Sensor.TYPE_GRAVITY);
    }

    private float dp(float value) {
        return value * getResources().getDisplayMetrics().density;
    }

    public void setListener(Listener listener) {
        this.listener = listener;
    }

    public void setGrid(boolean on) {
        grid = on;
        invalidate();
    }

    public void setLevel(boolean on) {
        level = on;
        invalidate();
    }

    public void setExposure(float ev) {
        this.ev = ev;
        showEv = true;
        invalidate();
    }

    public void hideExposure() {
        showEv = false;
        invalidate();
    }

    /** Draw these boxes as face frames. */
    public void setFaces(java.util.ArrayList<RectF> boxes) {
        faces.clear();
        if (boxes != null) {
            faces.addAll(boxes);
        }
        invalidate();
    }

    public void clearFocus() {
        focusX = -1;
        invalidate();
    }

    @Override
    protected void onAttachedToWindow() {
        super.onAttachedToWindow();
        if (gravity != null) {
            sensors.registerListener(this, gravity, SensorManager.SENSOR_DELAY_UI);
        }
    }

    @Override
    protected void onDetachedFromWindow() {
        if (sensors != null) {
            sensors.unregisterListener(this);
        }
        super.onDetachedFromWindow();
    }

    @Override
    public void onSensorChanged(SensorEvent event) {
        if (event.sensor.getType() != Sensor.TYPE_GRAVITY) {
            return;
        }
        // The phone is level when gravity points straight down the z axis; the roll is
        // how far the horizon is rotated, which is what a spirit level shows.
        float gx = event.values[0];
        float gy = event.values[1];
        float gz = event.values[2];
        roll = (float) Math.toDegrees(Math.atan2(gx, gy));
        tilt = Math.abs(gz);
        haveRoll = true;
        if (level) {
            invalidate();
        }
    }

    @Override
    public void onAccuracyChanged(Sensor sensor, int accuracy) {
    }

    @Override
    protected void onDraw(Canvas canvas) {
        super.onDraw(canvas);
        float w = getWidth();
        float h = getHeight();

        if (grid) {
            for (int i = 1; i < 3; i++) {
                float x = w * i / 3f;
                float y = h * i / 3f;
                canvas.drawLine(x, 0, x, h, line);
                canvas.drawLine(0, y, w, y, line);
            }
        }

        if (level && haveRoll) {
            // A horizon line that turns with the phone, plus a tick that stays put: they
            // line up when the picture will be level.
            float cx = w / 2f;
            float cy = h / 2f;
            float half = Math.min(w, h) * 0.28f;
            canvas.save();
            canvas.rotate(roll, cx, cy);
            boolean ok = Math.abs(roll) < 1.0f;
            line.setColor(ok ? 0xCC7CFC9B : 0x88FFFFFF);
            line.setStrokeWidth(dp(ok ? 2f : 1f));
            canvas.drawLine(cx - half, cy, cx + half, cy, line);
            canvas.restore();
            line.setColor(0x66FFFFFF);
            line.setStrokeWidth(dp(1f));
            canvas.drawLine(cx - half * 0.35f, cy, cx + half * 0.35f, cy, line);
        }

        for (int i = 0; i < faces.size(); i++) {
            RectF face = faces.get(i);
            if (face.width() < dp(8)) {
                continue;
            }
            box.setColor(0xCC5ED6EC);
            canvas.drawRoundRect(face, dp(10), dp(10), box);
        }
        box.setColor(0xCCFFD166);

        if (focusX >= 0) {
            float size = dp(72);
            RectF rect = new RectF(focusX - size / 2, focusY - size / 2,
                    focusX + size / 2, focusY + size / 2);
            canvas.drawRoundRect(rect, dp(8), dp(8), box);
            if (showEv) {
                drawExposure(canvas, focusX, focusY, size);
            }
        }
    }

    private void drawExposure(Canvas canvas, float cx, float cy, float size) {
        float barHeight = size * 1.6f;
        float top = cy - barHeight / 2;
        float right = cx + size / 2 + dp(18);
        canvas.drawRect(right - dp(3), top, right + dp(3), top + barHeight, shade);
        float centre = top + barHeight / 2;
        float fraction = Math.max(-1f, Math.min(1f, ev / 3f));
        float knob = centre - fraction * barHeight / 2;
        box.setColor(Color.WHITE);
        canvas.drawCircle(right, knob, dp(7), box);
        box.setColor(0xCCFFD166);
    }

    @Override
    public boolean onTouchEvent(MotionEvent event) {
        switch (event.getActionMasked()) {
            case MotionEvent.ACTION_DOWN:
                focusX = event.getX();
                focusY = event.getY();
                showEv = true;
                if (listener != null) {
                    listener.onFocus(focusX, focusY);
                }
                invalidate();
                return true;
            case MotionEvent.ACTION_MOVE: {
                // Dragging up and down next to the box is the exposure gesture everyone
                // expects from a phone camera.
                float dy = focusY - event.getY();
                float next = Math.max(-3f, Math.min(3f, dy / dp(180) * 6f));
                if (Math.abs(next - ev) > 0.05f) {
                    ev = next;
                    if (listener != null) {
                        listener.onExposure(ev);
                    }
                    invalidate();
                }
                return true;
            }
            case MotionEvent.ACTION_UP:
            case MotionEvent.ACTION_CANCEL:
                return true;
            default:
                return super.onTouchEvent(event);
        }
    }
}

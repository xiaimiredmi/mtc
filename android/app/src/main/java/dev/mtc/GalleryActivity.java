package dev.mtc;

import android.app.Activity;
import android.content.Intent;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.util.LruCache;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.app.AlertDialog;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.provider.OpenableColumns;
import android.widget.BaseAdapter;
import android.widget.Button;
import android.widget.FrameLayout;
import android.widget.GridView;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

import java.io.File;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * The launcher screen: a grid of the pictures this app has taken, and one button
 * to take another.
 *
 * <p>Thumbnails are decoded by the codec at a bounded size (the codec supports
 * decoding straight to a smaller picture, so a 12 MP file does not become a
 * 12 MP bitmap just to fill a cell), on a small thread pool, and kept in an
 * {@link LruCache} sized to an eighth of the heap.
 */
public class GalleryActivity extends Activity {
    private static final int REQ_IMPORT = 7;
    // One at a time: decoding a photo peaks at tens of megabytes, and three grid
    // cells decoding at once is how the app got killed on a phone. The results are
    // cached next to the pictures, so this costs a file once and nothing after.
    private final ExecutorService pool = Executors.newFixedThreadPool(1);
    private final Handler ui = new Handler(Looper.getMainLooper());
    private final List<File> files = new ArrayList<>();
    private LruCache<String, Bitmap> cache;
    private GridView grid;
    private TextView stats;
    private Adapter adapter;

    @Override
    protected void onCreate(Bundle state) {
        super.onCreate(state);

        int cacheBytes = (int) (Runtime.getRuntime().maxMemory() / 8);
        cache = new LruCache<String, Bitmap>(cacheBytes) {
            @Override
            protected int sizeOf(String key, Bitmap value) {
                return value.getByteCount();
            }
        };

        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setBackgroundColor(0xFF111114);

        LinearLayout head = new LinearLayout(this);
        head.setOrientation(LinearLayout.VERTICAL);
        head.setPadding(dp(20), dp(18), dp(20), dp(10));
        TextView title = new TextView(this);
        title.setText("MTC 相册");
        title.setTextColor(0xFFF2F2F4);
        title.setTextSize(22);
        head.addView(title);
        stats = new TextView(this);
        stats.setTextColor(0xFF8C8C93);
        stats.setTextSize(12);
        stats.setPadding(0, dp(4), 0, 0);
        head.addView(stats);
        root.addView(head, new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));

        grid = new GridView(this);
        grid.setNumColumns(3);
        grid.setHorizontalSpacing(dp(2));
        grid.setVerticalSpacing(dp(2));
        grid.setPadding(dp(2), dp(2), dp(2), dp(2));
        grid.setBackgroundColor(0xFF111114);
        adapter = new Adapter();
        grid.setAdapter(adapter);
        grid.setOnItemClickListener(new android.widget.AdapterView.OnItemClickListener() {
            @Override
            public void onItemClick(android.widget.AdapterView<?> parent, View view, int position, long id) {
                Intent i = new Intent(GalleryActivity.this, ViewerActivity.class);
                i.putExtra(ViewerActivity.EXTRA_PATH, files.get(position).getAbsolutePath());
                startActivity(i);
            }
        });
        grid.setOnItemLongClickListener(new android.widget.AdapterView.OnItemLongClickListener() {
            @Override
            public boolean onItemLongClick(android.widget.AdapterView<?> parent, View view,
                                           final int position, long id) {
                final File file = files.get(position);
                new AlertDialog.Builder(GalleryActivity.this)
                        .setTitle(file.getName())
                        .setMessage(Store.human(file.length()) + "，要删除吗？")
                        .setNegativeButton("取消", null)
                        .setPositiveButton("删除", new android.content.DialogInterface.OnClickListener() {
                            @Override
                            public void onClick(android.content.DialogInterface d, int which) {
                                //noinspection ResultOfMethodCallIgnored
                                Store.deleteWithThumbnail(file);
                                cache.remove(file.getAbsolutePath() + "@" + file.lastModified());
                                reload();
                            }
                        })
                        .show();
                return true;
            }
        });
        root.addView(grid, new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f));

        FrameLayout bar = new FrameLayout(this);
        bar.setBackgroundColor(0xFF1B1B1F);
        Button shoot = new Button(this);
        shoot.setText("拍照");
        shoot.setAllCaps(false);
        shoot.setTextSize(15);
        FrameLayout.LayoutParams lp = new FrameLayout.LayoutParams(dp(140), dp(46),
                Gravity.CENTER);
        lp.setMargins(0, dp(10), 0, dp(10));
        bar.addView(shoot, lp);
        shoot.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                startActivity(new Intent(GalleryActivity.this, CameraActivity.class));
            }
        });

        TextView importBtn = new TextView(this);
        importBtn.setText("导入");
        importBtn.setTextColor(0xFFB9B9C0);
        importBtn.setTextSize(13);
        importBtn.setPadding(dp(16), dp(10), dp(16), dp(10));
        bar.addView(importBtn, new FrameLayout.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT,
                Gravity.CENTER_VERTICAL | Gravity.END));
        importBtn.setOnClickListener(new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                Intent pick = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                pick.addCategory(Intent.CATEGORY_OPENABLE);
                pick.setType("*/*");
                try {
                    startActivityForResult(pick, REQ_IMPORT);
                } catch (Exception e) {
                    Toast.makeText(GalleryActivity.this, "没有可用的文件选择器",
                            Toast.LENGTH_SHORT).show();
                }
            }
        });
        root.addView(bar, new LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));

        setContentView(root);
        Mtc.warmUp();
    }

    /**
     * Copy a picked file into the app's own folder and show it.
     *
     * <p>This is how a `.mtc` taken elsewhere - or pulled off a computer - gets
     * into the gallery without asking for storage permission: the system picker
     * grants access to exactly one file.
     */
    @Override
    protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        if (request != REQ_IMPORT || result != RESULT_OK || data == null || data.getData() == null) {
            return;
        }
        Uri uri = data.getData();
        String name = "IMPORT_" + System.currentTimeMillis() + ".mtc";
        Cursor cursor = null;
        try {
            cursor = getContentResolver().query(uri,
                    new String[]{OpenableColumns.DISPLAY_NAME}, null, null, null);
            if (cursor != null && cursor.moveToFirst()) {
                String shown = cursor.getString(0);
                if (shown != null && shown.length() > 0) {
                    name = shown;
                }
            }
        } catch (Exception ignored) {
            // keep the generated name
        } finally {
            if (cursor != null) {
                cursor.close();
            }
        }
        if (!name.toLowerCase(Locale.ROOT).endsWith(".mtc")) {
            name = name + ".mtc";
        }
        try {
            InputStream in = getContentResolver().openInputStream(uri);
            java.io.FileOutputStream out =
                    new java.io.FileOutputStream(new File(Store.dir(this), name));
            byte[] buf = new byte[64 * 1024];
            int n;
            try {
                while ((n = in.read(buf)) > 0) {
                    out.write(buf, 0, n);
                }
            } finally {
                out.close();
                in.close();
            }
            Toast.makeText(this, "已导入 " + name, Toast.LENGTH_SHORT).show();
            reload();
        } catch (Exception e) {
            Toast.makeText(this, "导入失败：" + e.getMessage(), Toast.LENGTH_LONG).show();
        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        reload();
    }

    private void reload() {
        files.clear();
        files.addAll(Store.list(this));
        adapter.notifyDataSetChanged();
        long bytes = Store.totalBytes(files);
        stats.setText(String.format(Locale.US, "%d 张 · %s · %s",
                files.size(), Store.human(bytes), Store.dir(this).getAbsolutePath()) + Store.publicNote(this));
        if (files.isEmpty()) {
            Toast.makeText(this, "还没有照片，点下面的“拍照”", Toast.LENGTH_SHORT).show();
        }
    }

    @Override
    protected void onDestroy() {
        super.onDestroy();
        pool.shutdownNow();
    }

    /**
     * Thumbnail from the sidecar when there is one, otherwise decode the picture,
     * keep the result as a sidecar, and use that next time.
     */
    private Bitmap loadThumb(File mtc, int side) {
        if (Store.hasThumbnail(mtc)) {
            BitmapFactory.Options bounds = new BitmapFactory.Options();
            bounds.inJustDecodeBounds = true;
            BitmapFactory.decodeFile(Store.thumbFile(mtc).getAbsolutePath(), bounds);
            BitmapFactory.Options opts = new BitmapFactory.Options();
            int scale = 1;
            while (bounds.outWidth / (scale * 2) >= side && bounds.outHeight / (scale * 2) >= side) {
                scale *= 2;
            }
            opts.inSampleSize = scale;
            Bitmap cached = BitmapFactory.decodeFile(
                    Store.thumbFile(mtc).getAbsolutePath(), opts);
            if (cached != null) {
                return cached;
            }
        }
        Store.writeThumbnail(mtc, Math.max(320, side));
        return Store.hasThumbnail(mtc)
                ? BitmapFactory.decodeFile(Store.thumbFile(mtc).getAbsolutePath())
                : Mtc.thumbnail(mtc, side);
    }

    private int dp(int v) {
        return Math.round(v * getResources().getDisplayMetrics().density);
    }

    private class Adapter extends BaseAdapter {
        @Override
        public int getCount() {
            return files.size();
        }

        @Override
        public Object getItem(int position) {
            return files.get(position);
        }

        @Override
        public long getItemId(int position) {
            return position;
        }

        @Override
        public View getView(int position, View reuse, ViewGroup parent) {
            ImageView view = reuse instanceof ImageView ? (ImageView) reuse : new ImageView(GalleryActivity.this);
            view.setScaleType(ImageView.ScaleType.CENTER_CROP);
            view.setBackgroundColor(0xFF1E1E22);
            File file = files.get(position);
            String key = file.getAbsolutePath() + "@" + file.lastModified();
            Bitmap cached = cache.get(key);
            view.setTag(key);
            if (cached != null) {
                view.setImageBitmap(cached);
                return view;
            }
            view.setImageDrawable(null);
            final int side = Math.max(120, parent.getWidth() / 3);
            final ImageView target = view;
            pool.execute(new Runnable() {
                @Override
                public void run() {
                    final Bitmap bmp = loadThumb(file, side);
                    if (bmp == null) {
                        return;
                    }
                    cache.put(key, bmp);
                    ui.post(new Runnable() {
                        @Override
                        public void run() {
                            if (key.equals(target.getTag())) {
                                target.setImageBitmap(bmp);
                            }
                        }
                    });
                }
            });
            return view;
        }
    }
}

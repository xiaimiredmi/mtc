package dev.mtc;

import android.content.ContentResolver;
import android.content.Context;
import android.database.Cursor;
import android.net.Uri;
import android.os.Build;
import android.provider.MediaStore;
import android.os.Environment;

import java.io.File;
import java.io.FileFilter;
import java.io.FileOutputStream;
import java.io.IOException;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.Date;
import java.util.List;
import java.util.Locale;

/**
 * Where the pictures live.
 *
 * <p>App-specific external storage: no permission needed on any supported
 * Android version, visible over USB at
 * {@code Android/data/dev.mtc/files/Pictures}, and deleted with the app. That
 * keeps the camera usable without asking for storage access first.
 */
public final class Store {
    private Store() {
    }

    public static File dir(Context context) {
        File base = context.getExternalFilesDir(Environment.DIRECTORY_PICTURES);
        if (base == null) {
            base = new File(context.getFilesDir(), "Pictures");
        }
        if (!base.exists()) {
            //noinspection ResultOfMethodCallIgnored
            base.mkdirs();
        }
        return base;
    }

    /** Newest first. */
    public static List<File> list(Context context) {
        File[] files = dir(context).listFiles(new FileFilter() {
            @Override
            public boolean accept(File file) {
                return file.isFile()
                        && file.getName().toLowerCase(Locale.ROOT).endsWith(".mtc");
            }
        });
        List<File> out = new ArrayList<File>();
        if (files != null) {
            Collections.addAll(out, files);
        }
        Collections.sort(out, new Comparator<File>() {
            @Override
            public int compare(File a, File b) {
                return Long.compare(b.lastModified(), a.lastModified());
            }
        });
        return out;
    }

    public static long totalBytes(List<File> files) {
        long n = 0;
        for (int i = 0; i < files.size(); i++) {
            n += files.get(i).length();
        }
        return n;
    }

    public static File nextPhoto(Context context) {
        String stamp = new SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(new Date());
        return new File(dir(context), "IMG_" + stamp + ".mtc");
    }

    public static boolean write(File file, byte[] data) {
        try {
            FileOutputStream out = new FileOutputStream(file);
            try {
                out.write(data);
                out.getFD().sync();
            } finally {
                out.close();
            }
            return true;
        } catch (IOException e) {
            return false;
        }
    }

    /**
     * The cached thumbnail for a picture.
     *
     * <p>Decoding a 12 MP file takes half a second; doing that for every cell of
     * the grid every time it is opened is not acceptable. The first view writes a
     * small PNG next to the picture and every later view reads that instead.
     */
    public static File thumbFile(File mtc) {
        return new File(mtc.getAbsolutePath() + ".thumb.png");
    }

    /** Build (or refresh) the cached thumbnail; returns false if it could not. */
    public static boolean writeThumbnail(File mtc, int side) {
        File thumb = thumbFile(mtc);
        android.graphics.Bitmap bmp = Mtc.thumbnail(mtc, side);
        if (bmp == null) {
            return false;
        }
        try {
            FileOutputStream out = new FileOutputStream(thumb);
            try {
                return bmp.compress(android.graphics.Bitmap.CompressFormat.PNG, 95, out);
            } finally {
                out.close();
                bmp.recycle();
            }
        } catch (IOException e) {
            return false;
        }
    }

    /** True when the cached thumbnail is at least as new as the picture. */
    public static boolean hasThumbnail(File mtc) {
        File thumb = thumbFile(mtc);
        return thumb.exists() && thumb.lastModified() >= mtc.lastModified();
    }

    public static void deleteWithThumbnail(File mtc) {
        //noinspection ResultOfMethodCallIgnored
        thumbFile(mtc).delete();
        //noinspection ResultOfMethodCallIgnored
        mtc.delete();
    }


    /**
     * A short note about the copies that live outside the app.
     *
     * <p>Pictures are written twice: here for the gallery to browse, and into DCIM/MTC
     * through MediaStore so other apps and a PC can open them. The album header says how
     * many of those public copies exist, because "where did my photo go" is a question
     * the path alone does not answer.
     */
    public static String publicNote(Context context) {
        if (Build.VERSION.SDK_INT < 29) {
            return "";
        }
        Cursor cursor = null;
        try {
            ContentResolver resolver = context.getContentResolver();
            cursor = resolver.query(
                    MediaStore.Files.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY),
                    new String[]{ MediaStore.MediaColumns._ID },
                    MediaStore.MediaColumns.RELATIVE_PATH + " LIKE ?",
                    new String[]{ "DCIM/MTC%" },
                    null);
            int count = cursor == null ? 0 : cursor.getCount();
            return count > 0 ? " · 公共 DCIM/MTC " + count + " 张" : "";
        } catch (Exception e) {
            return "";
        } finally {
            if (cursor != null) {
                cursor.close();
            }
        }
    }
    /**
     * One picture, wherever it happens to live.
     *
     * <p>A picture is in the public album (DCIM/MTC, through MediaStore) or in the app's
     * own folder, and the gallery has to cope with both without asking which. Exactly one
     * of `file` and `uri` is set.
     */
    public static final class Photo {
        public final String name;
        public final long size;
        public final long modified;
        public final File file;
        public final Uri uri;

        Photo(String name, long size, long modified, File file, Uri uri) {
            this.name = name;
            this.size = size;
            this.modified = modified;
            this.file = file;
            this.uri = uri;
        }

        public boolean isPublic() {
            return uri != null;
        }
    }

    /** Every `.mtc` the app can see: the public album first, then its own folder. */
    public static List<Photo> listPhotos(Context context) {
        List<Photo> out = new ArrayList<Photo>();
        List<String> seen = new ArrayList<String>();
        if (Build.VERSION.SDK_INT >= 29) {
            Cursor cursor = null;
            try {
                ContentResolver resolver = context.getContentResolver();
                cursor = resolver.query(
                        MediaStore.Files.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY),
                        new String[]{
                                MediaStore.MediaColumns._ID,
                                MediaStore.MediaColumns.DISPLAY_NAME,
                                MediaStore.MediaColumns.SIZE,
                                MediaStore.MediaColumns.DATE_MODIFIED },
                        MediaStore.MediaColumns.RELATIVE_PATH + " LIKE ?",
                        new String[]{ "DCIM/MTC%" },
                        MediaStore.MediaColumns.DATE_MODIFIED + " DESC");
                while (cursor != null && cursor.moveToNext()) {
                    String name = cursor.getString(1);
                    if (name == null || !name.endsWith(".mtc")) {
                        continue;
                    }
                    long id = cursor.getLong(0);
                    Uri uri = android.content.ContentUris.withAppendedId(
                            MediaStore.Files.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY), id);
                    // MediaStore keeps DATE_MODIFIED in seconds.
                    out.add(new Photo(name, cursor.getLong(2), cursor.getLong(3) * 1000L, null, uri));
                    seen.add(name);
                }
            } catch (Exception ignored) {
                // A locked-down container has no MediaStore access; the app folder still works.
            } finally {
                if (cursor != null) {
                    cursor.close();
                }
            }
        }
        File[] files = dir(context).listFiles(new FileFilter() {
            @Override
            public boolean accept(File file) {
                return file.isFile() && file.getName().endsWith(".mtc");
            }
        });
        if (files != null) {
            for (int i = 0; i < files.length; i++) {
                File file = files[i];
                // The public copy is the same picture; showing both would double the grid.
                if (seen.contains(file.getName())) {
                    continue;
                }
                out.add(new Photo(file.getName(), file.length(), file.lastModified(), file, null));
            }
        }
        Collections.sort(out, new Comparator<Photo>() {
            @Override
            public int compare(Photo a, Photo b) {
                return Long.compare(b.modified, a.modified);
            }
        });
        return out;
    }

    public static byte[] read(Context context, Photo photo) {
        if (photo.file != null) {
            return Mtc.read(photo.file);
        }
        try {
            java.io.InputStream in = context.getContentResolver().openInputStream(photo.uri);
            if (in == null) {
                return null;
            }
            try {
                java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
                byte[] buffer = new byte[64 * 1024];
                int n;
                while ((n = in.read(buffer)) > 0) {
                    out.write(buffer, 0, n);
                }
                return out.toByteArray();
            } finally {
                in.close();
            }
        } catch (Exception e) {
            return null;
        }
    }

    /** A file the rest of the app (and the viewer) can open by path. */
    public static File materialise(Context context, Photo photo) {
        if (photo.file != null) {
            return photo.file;
        }
        File cache = new File(context.getCacheDir(), "public");
        if (!cache.exists()) {
            cache.mkdirs();
        }
        File out = new File(cache, photo.name);
        if (out.exists() && out.length() == photo.size) {
            return out;
        }
        byte[] bytes = read(context, photo);
        if (bytes == null) {
            return null;
        }
        try {
            FileOutputStream fos = new FileOutputStream(out);
            try {
                fos.write(bytes);
            } finally {
                fos.close();
            }
            return out;
        } catch (IOException e) {
            return null;
        }
    }

    /** Cached thumbnail for a picture, wherever it lives. */
    public static android.graphics.Bitmap thumbnail(Context context, Photo photo, int side) {
        File cache = new File(context.getCacheDir(), "thumbs");
        if (!cache.exists()) {
            cache.mkdirs();
        }
        File thumb = new File(cache, photo.name + "." + side + ".png");
        if (thumb.exists() && thumb.lastModified() >= photo.modified) {
            return android.graphics.BitmapFactory.decodeFile(thumb.getAbsolutePath());
        }
        byte[] bytes = read(context, photo);
        if (bytes == null) {
            return null;
        }
        android.graphics.Bitmap bmp = Mtc.thumbnail(bytes, side);
        if (bmp == null) {
            return null;
        }
        try {
            FileOutputStream out = new FileOutputStream(thumb);
            try {
                bmp.compress(android.graphics.Bitmap.CompressFormat.PNG, 90, out);
            } finally {
                out.close();
            }
        } catch (IOException ignored) {
        }
        return bmp;
    }

    public static boolean deletePhoto(Context context, Photo photo) {
        boolean ok = false;
        try {
            if (photo.uri != null) {
                ok = context.getContentResolver().delete(photo.uri, null, null) > 0;
            }
            if (photo.file != null) {
                ok = photo.file.delete() || ok;
            }
            // The copy on the other side, if the picture was written twice before.
            File twin = new File(dir(context), photo.name);
            if (twin.exists() && !twin.equals(photo.file)) {
                twin.delete();
                thumbFile(twin).delete();
            }
        } catch (Exception ignored) {
        }
        return ok;
    }    public static String human(long bytes) {
        if (bytes >= 1024L * 1024L) {
            return String.format(Locale.US, "%.2f MB", bytes / 1048576.0);
        }
        if (bytes >= 1024L) {
            return String.format(Locale.US, "%.1f KB", bytes / 1024.0);
        }
        return bytes + " B";
    }
}

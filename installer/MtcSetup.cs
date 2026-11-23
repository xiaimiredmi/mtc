// MTC Windows installer.
//
// One .exe, no dependencies: the binaries ride along as an embedded zip and the
// whole thing is compiled by the C# compiler that ships with .NET Framework, so
// building it needs nothing installed.
//
//   MtcSetup.exe                     the wizard
//   MtcSetup.exe /S                  silent install with the defaults
//   MtcSetup.exe /dir "D:\Tools\MTC" where to put it (works with /S too)
//   MtcSetup.exe /nopath /noassoc /nostartmenu /nodesktop   skip pieces
//   MtcSetup.exe /extract "D:\tmp"   just unpack, change nothing
//   MtcSetup.exe /uninstall [/S]     remove it again
//
// Everything is per-user: HKCU and %LOCALAPPDATA%, so no administrator prompt and
// no surprises for other accounts on the machine.

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text;
using System.Windows.Forms;
using Microsoft.Win32;

[assembly: AssemblyTitle("MTC installer")]
[assembly: AssemblyDescription("MTC image codec, viewer and command line tools")]
[assembly: AssemblyProduct("MTC")]
[assembly: AssemblyCompany("xiaimiredmi")]
[assembly: AssemblyVersion("1.6.0.0")]
[assembly: AssemblyFileVersion("1.6.0.0")]

namespace MtcSetup
{
    internal sealed class Options
    {
        public string Dir;
        public bool Silent;
        public bool Path = true;
        public bool Assoc = true;
        public bool StartMenu = true;
        public bool Desktop;
        public bool Uninstall;
        public string ExtractTo;
        public bool Okay = true;
    }

    internal static class Program
    {
        internal const string AppName = "MTC";
        internal const string Version = "1.6.0";
        internal const string Publisher = "xiaimiredmi";
        internal const string UninstallKey = @"Software\Microsoft\Windows\CurrentVersion\Uninstall\MTC";
        internal const string ClassKey = @"Software\Classes\MTC.Image";
        internal const string ExtKey = @"Software\Classes\.mtc";

        internal static string DefaultDir
        {
            get
            {
                return Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                    @"Programs\MTC");
            }
        }

        [STAThread]
        private static int Main(string[] rawArgs)
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);

            // Add/Remove Programs runs the copy we installed as uninstall.exe. Its
            // name is the intent, and it has to be known before the arguments are
            // parsed: an uninstall without /dir reads the install location from the
            // registry, and getting that wrong sends the files to the wrong folder.
            bool byName = string.Equals(Path.GetFileNameWithoutExtension(Application.ExecutablePath),
                "uninstall", StringComparison.OrdinalIgnoreCase);

            Options options = Parse(rawArgs, byName);
            if (!options.Okay) return 2;

            if (!string.IsNullOrEmpty(options.ExtractTo))
            {
                try
                {
                    Payload.ExtractTo(options.ExtractTo);
                    Console.WriteLine("extracted to " + options.ExtractTo);
                    return 0;
                }
                catch (Exception e)
                {
                    Console.Error.WriteLine(e.Message);
                    return 1;
                }
            }

            if (options.Uninstall) return Uninstaller.Run(options);

            if (options.Silent)
            {
                var log = new StringBuilder();
                try
                {
                    Installer.Install(options, line => log.AppendLine(line));
                    Console.WriteLine(log.ToString());
                    return 0;
                }
                catch (Exception e)
                {
                    Console.Error.WriteLine(e.Message);
                    Console.Error.WriteLine(log.ToString());
                    return 1;
                }
            }

            Application.Run(new SetupForm(options));
            return 0;
        }

        private static Options Parse(string[] args, bool uninstallByName)
        {
            var options = new Options { Dir = DefaultDir, Uninstall = uninstallByName };
            for (int i = 0; i < args.Length; i++)
            {
                string a = args[i].ToLowerInvariant().TrimStart('-', '/');
                switch (a)
                {
                    case "s":
                    case "silent":
                    case "quiet":
                        options.Silent = true;
                        break;
                    case "dir":
                    case "d":
                        if (i + 1 < args.Length) options.Dir = args[++i];
                        break;
                    case "nopath": options.Path = false; break;
                    case "noassoc": options.Assoc = false; break;
                    case "nostartmenu": options.StartMenu = false; break;
                    case "desktop": options.Desktop = true; break;
                    case "nodesktop": options.Desktop = false; break;
                    case "uninstall":
                    case "u":
                        options.Uninstall = true;
                        break;
                    case "extract":
                        if (i + 1 < args.Length) options.ExtractTo = args[++i];
                        break;
                    case "?":
                    case "help":
                        Console.WriteLine("MtcSetup.exe [/S] [/dir <path>] [/nopath] [/noassoc] [/nostartmenu] [/desktop]");
                        Console.WriteLine("MtcSetup.exe /extract <dir>");
                        Console.WriteLine("MtcSetup.exe /uninstall [/S]");
                        return new Options { Okay = false };
                    default:
                        Console.Error.WriteLine("unknown argument: " + args[i]);
                        return new Options { Okay = false };
                }
            }
            // An uninstall must know where the files went, so read it back from the
            // uninstall entry unless a directory was given explicitly.
            if (options.Uninstall && options.Dir == DefaultDir)
            {
                string installed = Installer.InstalledDir();
                if (installed != null) options.Dir = installed;
            }
            return options;
        }
    }

    /// <summary>
    /// The binaries, compiled into this executable.
    ///
    /// One resource per file, deflated with DeflateStream: that type lives in
    /// System.dll and is available to the compiler that ships with .NET Framework,
    /// whereas ZipArchive would drag in another assembly that is not always
    /// resolvable by simple name.
    /// </summary>
    internal static class Payload
    {
        internal static readonly string[] Files =
        {
            "mtcodec.exe", "mtbench.exe", "mtview.exe", "mtc.dll", "mtc.h", "mtc.ico", "LICENSE", "README.txt"
        };

        internal const string ResourcePrefix = "payload.";

        internal static void ExtractTo(string directory)
        {
            Directory.CreateDirectory(directory);
            Assembly assembly = Assembly.GetExecutingAssembly();
            foreach (string file in Files)
            {
                string name = ResourcePrefix + file;
                using (Stream raw = assembly.GetManifestResourceStream(name))
                {
                    if (raw == null) continue;
                    using (var inflater = new DeflateStream(raw, CompressionMode.Decompress))
                    using (FileStream target = File.Create(Path.Combine(directory, file)))
                    {
                        inflater.CopyTo(target);
                    }
                }
            }
        }
    }

    internal static class Installer
    {
        internal static string InstalledDir()
        {
            using (RegistryKey key = Registry.CurrentUser.OpenSubKey(Program.UninstallKey))
            {
                if (key == null) return null;
                object value = key.GetValue("InstallLocation");
                return value as string;
            }
        }

        internal static void Install(Options options, Action<string> log)
        {
            string dir = Path.GetFullPath(options.Dir);
            log("安装位置：" + dir);

            try
            {
                Directory.CreateDirectory(dir);
            }
            catch (UnauthorizedAccessException)
            {
                throw new ApplicationException(
                    "没有权限写入 " + dir + "。\n请换一个目录（默认的 " + Program.DefaultDir +
                    " 不需要管理员权限），或者右键以管理员身份运行。");
            }

            Payload.ExtractTo(dir);
            foreach (string file in Payload.Files)
            {
                string path = Path.Combine(dir, file);
                if (File.Exists(path)) log("  " + file + "  " + Size(new FileInfo(path).Length));
            }

            // The uninstaller is this executable, copied in place so that the
            // Add/Remove Programs entry can point at a file that lives with the app.
            string uninstaller = Path.Combine(dir, "uninstall.exe");
            File.Copy(Application.ExecutablePath, uninstaller, true);
            log("  卸载程序 uninstall.exe");

            if (options.Assoc) RegisterAssociation(dir, log); else RemoveAssociation(log);
            if (options.Path) PrependPath(dir, log); else RemovePathEntry(dir, log);
            WriteUninstallEntry(dir);
            if (options.StartMenu || options.Desktop) WriteShortcuts(dir, options, log);

            Shell.NotifyAssociationChanged();
            Shell.NotifyEnvironmentChanged();
            log("");
            log("完成。双击任意 .mtc 文件即可用 " + Program.AppName + " 查看。");
        }

        private static string Size(long bytes)
        {
            if (bytes >= 1024 * 1024) return (bytes / 1048576.0).ToString("0.0") + " MB";
            return (bytes / 1024) + " KB";
        }

        internal static void RegisterAssociation(string dir, Action<string> log)
        {
            using (RegistryKey ext = Registry.CurrentUser.CreateSubKey(Program.ExtKey))
            {
                ext.SetValue(null, "MTC.Image");
                ext.SetValue("Content Type", "image/x-mtc");
            }
            using (RegistryKey cls = Registry.CurrentUser.CreateSubKey(Program.ClassKey))
            {
                cls.SetValue(null, "MTC 图片");
                cls.SetValue("FriendlyTypeName", "MTC 图片");
                cls.SetValue("PerceivedType", "image");
            }
            using (RegistryKey icon = Registry.CurrentUser.CreateSubKey(Program.ClassKey + @"\DefaultIcon"))
            {
                string ico = Path.Combine(dir, "mtc.ico");
                icon.SetValue(null, File.Exists(ico) ? ico + ",0" : Path.Combine(dir, "mtview.exe") + ",0");
            }
            using (RegistryKey cmd = Registry.CurrentUser.CreateSubKey(Program.ClassKey + @"\shell\open\command"))
            {
                cmd.SetValue(null, "\"" + Path.Combine(dir, "mtview.exe") + "\" \"%1\"");
            }
            log("  已关联 .mtc（双击打开）");
        }

        internal static void RemoveAssociation(Action<string> log)
        {
            Registry.CurrentUser.DeleteSubKeyTree(Program.ExtKey, false);
            Registry.CurrentUser.DeleteSubKeyTree(Program.ClassKey, false);
            // The video container from older versions, if one was ever registered.
            Registry.CurrentUser.DeleteSubKeyTree(@"Software\Classes\.mtb", false);
            Registry.CurrentUser.DeleteSubKeyTree(@"Software\Classes\MTB.Image", false);
            Registry.CurrentUser.DeleteSubKeyTree(@"Software\Classes\MTB.Video", false);
            log("  已移除文件关联");
        }

        internal static void PrependPath(string dir, Action<string> log)
        {
            string current = ReadUserPath();
            var parts = new List<string>();
            foreach (string part in current.Split(';'))
            {
                string trimmed = part.Trim().TrimEnd('\\');
                if (trimmed.Length == 0) continue;
                if (string.Equals(trimmed, dir.TrimEnd('\\'), StringComparison.OrdinalIgnoreCase)) continue;
                parts.Add(part);
            }
            parts.Insert(0, dir.TrimEnd('\\'));
            WriteUserPath(string.Join(";", parts.ToArray()));
            log("  已加入 PATH（排在第一位）");
        }

        internal static void RemovePathEntry(string dir, Action<string> log)
        {
            string current = ReadUserPath();
            var keep = current.Split(';')
                .Where(p => p.Trim().Length > 0)
                .Where(p => !string.Equals(p.Trim().TrimEnd('\\'), dir.TrimEnd('\\'), StringComparison.OrdinalIgnoreCase))
                .ToArray();
            WriteUserPath(string.Join(";", keep));
            log("  已从 PATH 移除");
        }

        private static string ReadUserPath()
        {
            object value = Registry.GetValue(@"HKEY_CURRENT_USER\Environment", "Path", "");
            return value as string ?? "";
        }

        private static void WriteUserPath(string value)
        {
            using (RegistryKey env = Registry.CurrentUser.OpenSubKey("Environment", true))
            {
                if (env == null) return;
                env.SetValue("Path", value, RegistryValueKind.ExpandString);
            }
        }

        private static void WriteUninstallEntry(string dir)
        {
            using (RegistryKey key = Registry.CurrentUser.CreateSubKey(Program.UninstallKey))
            {
                key.SetValue("DisplayName", "MTC · 图片编解码器");
                key.SetValue("DisplayVersion", Program.Version);
                key.SetValue("Publisher", Program.Publisher);
                key.SetValue("InstallLocation", dir);
                key.SetValue("DisplayIcon", Path.Combine(dir, "mtview.exe") + ",0");
                key.SetValue("UninstallString", "\"" + Path.Combine(dir, "uninstall.exe") + "\" /uninstall");
                key.SetValue("QuietUninstallString", "\"" + Path.Combine(dir, "uninstall.exe") + "\" /uninstall /S");
                key.SetValue("NoModify", 1, RegistryValueKind.DWord);
                key.SetValue("NoRepair", 1, RegistryValueKind.DWord);
                long size = 0;
                foreach (string file in Payload.Files)
                {
                    string path = Path.Combine(dir, file);
                    if (File.Exists(path)) size += new FileInfo(path).Length;
                }
                key.SetValue("EstimatedSize", (int)(size / 1024), RegistryValueKind.DWord);
            }
        }

        private static void WriteShortcuts(string dir, Options options, Action<string> log)
        {
            string menu = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.Programs), Program.AppName);
            if (options.StartMenu)
            {
                Directory.CreateDirectory(menu);
                Shell.Shortcut(Path.Combine(menu, "MTC 查看器.lnk"), Path.Combine(dir, "mtview.exe"), "",
                    dir, "用 MTC 查看图片");
                Shell.Shortcut(Path.Combine(menu, "MTC 命令行.lnk"), Path.Combine(Environment.SystemDirectory, "cmd.exe"),
                    "/k cd /d \"" + dir + "\"", dir, "打开带 MTC 工具的终端");
                Shell.Shortcut(Path.Combine(menu, "卸载 MTC.lnk"), Path.Combine(dir, "uninstall.exe"), "",
                    dir, "卸载 MTC");
                log("  已创建开始菜单快捷方式");
            }
            if (options.Desktop)
            {
                string desktop = Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory);
                Shell.Shortcut(Path.Combine(desktop, "MTC 查看器.lnk"), Path.Combine(dir, "mtview.exe"), "",
                    dir, "用 MTC 查看图片");
                log("  已创建桌面快捷方式");
            }
        }

        internal static void Uninstall(Options options, Action<string> log)
        {
            string dir = Path.GetFullPath(options.Dir);
            log("卸载：" + dir);

            RemoveAssociation(log);
            RemovePathEntry(dir, log);
            Registry.CurrentUser.DeleteSubKeyTree(Program.UninstallKey, false);
            log("  已移除卸载信息");

            string menu = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.Programs), Program.AppName);
            if (Directory.Exists(menu)) Directory.Delete(menu, true);
            string desktopLink = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory), "MTC 查看器.lnk");
            if (File.Exists(desktopLink)) File.Delete(desktopLink);
            log("  已删除快捷方式");

            Shell.NotifyAssociationChanged();
            Shell.NotifyEnvironmentChanged();

            // The last step cannot be done from inside this process: the uninstaller
            // is one of the files being deleted, and Windows will not delete a running
            // image. So a small batch file is left behind, it waits for this process
            // to exit and then removes the directory and itself. Doing the deletion
            // in-process as well only makes the race worse, so it is not attempted.
            if (!File.Exists(Path.Combine(dir, "mtview.exe")))
            {
                log("  目录里没有 mtview.exe，可能不是 MTC 的安装目录，已跳过删除文件");
                log("");
                log("已移除注册表项，文件未动：" + dir);
                return;
            }

            string bat = Path.Combine(Path.GetTempPath(), "mtc-cleanup-" + Guid.NewGuid().ToString("N") + ".bat");
            string script = string.Join(Environment.NewLine, new[]
            {
                "@echo off",
                "rem wait for the uninstaller to exit, then remove the install folder",
                "ping -n 3 127.0.0.1 >nul",
                "rmdir /s /q \"" + dir + "\"",
                "del /q \"%~f0\""
            });
            File.WriteAllText(bat, script, Encoding.Default);

            Process.Start(new ProcessStartInfo("cmd.exe", "/c \"" + bat + "\"")
            {
                WindowStyle = ProcessWindowStyle.Hidden,
                UseShellExecute = true
            });
            log("  安装目录将在几秒内删除：" + dir);
            log("");
            log("已卸载。");
        }
    }

    internal static class Uninstaller
    {
        internal static int Run(Options options)
        {
            if (!options.Silent)
            {
                DialogResult answer = MessageBox.Show(
                    "要卸载 " + Program.AppName + " 吗？\n\n" + options.Dir,
                    "MTC 卸载", MessageBoxButtons.OKCancel, MessageBoxIcon.Question);
                if (answer != DialogResult.OK) return 0;
            }
            var log = new StringBuilder();
            try
            {
                Installer.Uninstall(options, line =>
                {
                    log.AppendLine(line);
                    if (!options.Silent) Console.WriteLine(line);
                });
                if (options.Silent) Console.WriteLine(log.ToString());
                return 0;
            }
            catch (Exception e)
            {
                Console.Error.WriteLine(e.Message);
                if (!options.Silent)
                {
                    MessageBox.Show(e.Message, "卸载失败", MessageBoxButtons.OK, MessageBoxIcon.Error);
                }
                return 1;
            }
        }
    }

    internal static class Shell
    {
        private const int ShcneAssocChanged = 0x08000000;
        private const int ShcnfIdList = 0;

        [DllImport("shell32.dll", CharSet = CharSet.Auto, SetLastError = true)]
        private static extern void SHChangeNotify(int eventId, int flags, IntPtr item1, IntPtr item2);

        [DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
        private static extern IntPtr SendMessageTimeout(
            IntPtr hWnd, uint msg, UIntPtr wParam, string lParam, uint flags, uint timeout, out UIntPtr result);

        private static readonly IntPtr HwndBroadcast = (IntPtr)0xffff;
        private const uint WmSettingChange = 0x001A;
        private const uint SmtoAbortIfHung = 0x0002;

        /// <summary>Tell Explorer the file association changed, so it stops showing a stale icon.</summary>
        internal static void NotifyAssociationChanged()
        {
            try { SHChangeNotify(ShcneAssocChanged, ShcnfIdList, IntPtr.Zero, IntPtr.Zero); }
            catch { /* cosmetic only */ }
        }

        /// <summary>Tell running shells that the user environment changed (PATH).</summary>
        internal static void NotifyEnvironmentChanged()
        {
            try
            {
                UIntPtr result;
                SendMessageTimeout(HwndBroadcast, WmSettingChange, UIntPtr.Zero, "Environment",
                    SmtoAbortIfHung, 3000, out result);
            }
            catch { /* a new shell will pick the change up anyway */ }
        }

        /// <summary>Create a .lnk through the Windows Script Host, no dependencies.</summary>
        internal static void Shortcut(string link, string target, string arguments, string workingDir, string description)
        {
            try
            {
                Type shell = Type.GetTypeFromProgID("WScript.Shell");
                if (shell == null) return;
                object instance = Activator.CreateInstance(shell);
                object shortcut = shell.InvokeMember("CreateShortcut", BindingFlags.InvokeMethod, null, instance,
                    new object[] { link });
                Type type = shortcut.GetType();
                type.InvokeMember("TargetPath", BindingFlags.SetProperty, null, shortcut, new object[] { target });
                type.InvokeMember("Arguments", BindingFlags.SetProperty, null, shortcut, new object[] { arguments });
                type.InvokeMember("WorkingDirectory", BindingFlags.SetProperty, null, shortcut, new object[] { workingDir });
                type.InvokeMember("Description", BindingFlags.SetProperty, null, shortcut, new object[] { description });
                type.InvokeMember("IconLocation", BindingFlags.SetProperty, null, shortcut, new object[] { target + ",0" });
                type.InvokeMember("Save", BindingFlags.InvokeMethod, null, shortcut, null);
            }
            catch
            {
                // A shortcut failing is not worth aborting an install for.
            }
        }
    }

    /// <summary>The wizard. Deliberately one screen: choose a folder, tick three boxes, done.</summary>
    internal sealed class SetupForm : Form
    {
        private readonly Options options;
        private readonly TextBox dirBox = new TextBox();
        private readonly CheckBox assocBox = new CheckBox();
        private readonly CheckBox pathBox = new CheckBox();
        private readonly CheckBox menuBox = new CheckBox();
        private readonly CheckBox desktopBox = new CheckBox();
        private readonly Button installButton = new Button();
        private readonly Button cancelButton = new Button();
        private readonly Button openButton = new Button();
        private readonly TextBox logBox = new TextBox();
        private bool installed;

        internal SetupForm(Options options)
        {
            this.options = options;
            Text = "MTC 安装程序";
            ClientSize = new Size(620, 470);
            StartPosition = FormStartPosition.CenterScreen;
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            MinimizeBox = false;
            Font = new Font("Microsoft YaHei UI", 9f);
            BackColor = Color.FromArgb(250, 250, 252);

            var title = new Label
            {
                Text = "MTC · 图片编解码器",
                Font = new Font("Microsoft YaHei UI", 16f, FontStyle.Bold),
                ForeColor = Color.FromArgb(20, 20, 28),
                AutoSize = true,
                Location = new Point(24, 20)
            };
            var subtitle = new Label
            {
                Text = "版本 " + Program.Version + "　·　命令行工具 + 图片查看器 + .mtc 文件关联\n" +
                       "装到当前用户，不需要管理员权限。",
                ForeColor = Color.FromArgb(90, 94, 104),
                AutoSize = true,
                Location = new Point(26, 56)
            };

            var dirLabel = new Label { Text = "安装位置", AutoSize = true, Location = new Point(26, 112) };
            dirBox.Location = new Point(26, 134);
            dirBox.Width = 470;
            dirBox.Text = options.Dir;
            var browse = new Button { Text = "浏览…", Location = new Point(504, 132), Size = new Size(88, 27) };
            browse.Click += (s, e) =>
            {
                using (var dialog = new FolderBrowserDialog())
                {
                    dialog.Description = "选择安装位置";
                    dialog.SelectedPath = dirBox.Text;
                    if (dialog.ShowDialog(this) == DialogResult.OK) dirBox.Text = dialog.SelectedPath;
                }
            };

            assocBox.Text = "关联 .mtc 文件（双击用 MTC 打开）";
            assocBox.Checked = options.Assoc;
            assocBox.AutoSize = true;
            assocBox.Location = new Point(26, 176);

            pathBox.Text = "加入用户 PATH（在终端里直接敲 mtcodec）";
            pathBox.Checked = options.Path;
            pathBox.AutoSize = true;
            pathBox.Location = new Point(26, 202);

            menuBox.Text = "创建开始菜单快捷方式";
            menuBox.Checked = options.StartMenu;
            menuBox.AutoSize = true;
            menuBox.Location = new Point(26, 228);

            desktopBox.Text = "创建桌面快捷方式";
            desktopBox.Checked = options.Desktop;
            desktopBox.AutoSize = true;
            desktopBox.Location = new Point(26, 254);

            logBox.Location = new Point(26, 288);
            logBox.Size = new Size(566, 126);
            logBox.Multiline = true;
            logBox.ReadOnly = true;
            logBox.ScrollBars = ScrollBars.Vertical;
            logBox.BackColor = Color.FromArgb(16, 18, 24);
            logBox.ForeColor = Color.FromArgb(200, 208, 220);
            logBox.Font = new Font("Consolas", 8.5f);
            logBox.Visible = false;

            installButton.Text = "安装";
            installButton.Size = new Size(120, 34);
            installButton.Location = new Point(330, 424);
            installButton.BackColor = Color.FromArgb(36, 118, 240);
            installButton.ForeColor = Color.White;
            installButton.FlatStyle = FlatStyle.Flat;
            installButton.FlatAppearance.BorderSize = 0;
            installButton.Click += (s, e) => Install();

            cancelButton.Text = "取消";
            cancelButton.Size = new Size(88, 34);
            cancelButton.Location = new Point(460, 424);
            cancelButton.Click += (s, e) => Close();

            openButton.Text = "打开文件夹";
            openButton.Size = new Size(110, 34);
            openButton.Location = new Point(210, 424);
            openButton.Visible = false;
            openButton.Click += (s, e) =>
            {
                try { Process.Start("explorer.exe", "\"" + Path.GetFullPath(dirBox.Text) + "\""); }
                catch { }
            };

            Controls.AddRange(new Control[]
            {
                title, subtitle, dirLabel, dirBox, browse,
                assocBox, pathBox, menuBox, desktopBox,
                logBox, openButton, installButton, cancelButton
            });
        }

        private void Install()
        {
            if (installed) { Close(); return; }
            options.Dir = dirBox.Text.Trim();
            options.Assoc = assocBox.Checked;
            options.Path = pathBox.Checked;
            options.StartMenu = menuBox.Checked;
            options.Desktop = desktopBox.Checked;

            if (options.Dir.Length == 0)
            {
                MessageBox.Show("请填写安装位置。", "MTC", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }

            logBox.Visible = true;
            logBox.Clear();
            installButton.Enabled = false;
            cancelButton.Enabled = false;
            Application.DoEvents();

            try
            {
                Installer.Install(options, line =>
                {
                    logBox.AppendText(line + Environment.NewLine);
                    Application.DoEvents();
                });
                installed = true;
                installButton.Text = "完成";
                installButton.Enabled = true;
                cancelButton.Enabled = true;
                openButton.Visible = true;
                assocBox.Enabled = pathBox.Enabled = menuBox.Enabled = desktopBox.Enabled = false;
                dirBox.Enabled = false;
            }
            catch (Exception e)
            {
                logBox.AppendText(Environment.NewLine + "失败：" + e.Message + Environment.NewLine);
                MessageBox.Show(e.Message, "安装失败", MessageBoxButtons.OK, MessageBoxIcon.Error);
                installButton.Enabled = true;
                cancelButton.Enabled = true;
            }
        }
    }
}

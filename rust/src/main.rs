//! MTC command line: encode, decode and inspect `.mtc` images.

use std::env;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;

use mtc::container::{self, AqMode, Options, VERSION_LEGACY};
use mtc::imgio;

fn usage() -> ExitCode {
    eprintln!("mtcodec - MTC image codec");
    eprintln!();
    eprintln!("usage:");
    eprintln!("  mtcodec info <file.mtc>");
    eprintln!("  mtcodec encode <in.png> <out.mtc> [-q 1..100] [-s 420|444] [--lossless]");
    eprintln!("                 [--aq off|on|auto] [--v1]");
    eprintln!("  mtcodec decode <in.mtc> <out.png> [--max-side N] [--rect x,y,w,h]");
    eprintln!("      --max-side decodes straight to a smaller picture;");
    eprintln!("      --rect decodes one rectangle, skipping the transform elsewhere.");
    eprintln!();
    eprintln!("  -q     quality, higher is bigger and closer (default 85)");
    eprintln!("  -s     chroma subsampling for colour (default 420)");
    eprintln!("  --aq   adaptive quantisation: off (default), on, or auto");
    eprintln!("  --v1   write the legacy version 1 stream");
    eprintln!("  --lossless  spatial prediction, pixel-exact (modes 3 and 4)");
    ExitCode::from(2)
}

fn cmd_info(path: &str) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| format!("{}: {}", path, e))?;
    let h = container::probe(&data)?;
    println!("file     {}", path);
    println!("bytes    {}", h.bytes);
    println!("version  {}", h.version);
    println!("mode     {} ({})", h.mode, h.mode_name());
    println!("quality  {}", h.quality);
    println!("alpha    {}", h.has_alpha());
    println!("aq       {}", h.aq());
    println!("size     {}x{}", h.width, h.height);
    Ok(())
}

fn cmd_encode(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        return Err("encode needs <in.png> <out.mtc>".into());
    }
    let (input, output) = (&args[2], &args[3]);
    let mut opts = Options::default();
    let mut i = 4;
    while i < args.len() {
        match args[i].as_str() {
            "-q" if i + 1 < args.len() => {
                opts.quality = args[i + 1]
                    .parse::<u8>()
                    .map_err(|_| "bad -q value".to_string())?
                    .clamp(1, 100);
                i += 2;
            }
            "-s" if i + 1 < args.len() => {
                opts.subsample_420 = match args[i + 1].as_str() {
                    "420" => true,
                    "444" => false,
                    other => return Err(format!("unknown -s mode {}", other)),
                };
                i += 2;
            }
            "--aq" if i + 1 < args.len() => {
                opts.aq = AqMode::parse(&args[i + 1])?;
                i += 2;
            }
            "--lossless" => {
                opts.lossless = true;
                opts.aq = AqMode::Off;
                i += 1;
            }
            "--v1" => {
                opts.version = VERSION_LEGACY;
                opts.aq = AqMode::Off;
                i += 1;
            }
            other => return Err(format!("unknown argument: {}", other)),
        }
    }
    let img = imgio::load(input)?;
    let t0 = Instant::now();
    let blob = container::encode(&img, &opts)?;
    let enc_ms = t0.elapsed().as_secs_f64() * 1000.0;
    fs::write(output, &blob).map_err(|e| format!("{}: {}", output, e))?;
    let t1 = Instant::now();
    let back = container::decode(&blob)?;
    let dec_ms = t1.elapsed().as_secs_f64() * 1000.0;
    let bpp = blob.len() as f64 * 8.0 / (img.width * img.height) as f64;
    println!(
        "encoded {}x{} -> {} ({} bytes, {:.3} bpp, psnr {:.2} dB, {:.0} ms + {:.0} ms decode)",
        img.width,
        img.height,
        output,
        blob.len(),
        bpp,
        img.psnr_visible(&back),
        enc_ms,
        dec_ms
    );
    let h = container::probe(&blob)?;
    println!(
        "  mode {} ({}) version {} aq {}",
        h.mode,
        h.mode_name(),
        h.version,
        h.aq()
    );
    Ok(())
}

fn cmd_decode(args: &[String]) -> Result<(), String> {
    if args.len() < 4 {
        return Err("decode needs <in.mtc> <out.png>".into());
    }
    let (input, output) = (&args[2], &args[3]);
    let mut max_side = 0usize;
    let mut rect: Option<(usize, usize, usize, usize)> = None;
    let mut i = 4;
    while i < args.len() {
        if args[i] == "--max-side" && i + 1 < args.len() {
            max_side = args[i + 1]
                .parse()
                .map_err(|_| "bad --max-side".to_string())?;
            i += 2;
        } else if args[i] == "--rect" && i + 1 < args.len() {
            // x,y,w,h - decode one rectangle, skipping the transform for the rest.
            let parts: Vec<usize> = args[i + 1]
                .split(',')
                .map(|p| p.trim().parse::<usize>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "bad --rect, want x,y,w,h".to_string())?;
            if parts.len() != 4 {
                return Err("bad --rect, want x,y,w,h".into());
            }
            rect = Some((parts[0], parts[1], parts[2], parts[3]));
            i += 2;
        } else {
            return Err(format!("unknown argument: {}", args[i]));
        }
    }
    let data = fs::read(input).map_err(|e| format!("{}: {}", input, e))?;
    let t0 = Instant::now();
    // A bounded size decodes straight to it instead of building the full picture.
    let img = if let Some((x, y, w, h)) = rect {
        container::decode_region(&data, x, y, w, h)?
    } else if max_side > 0 {
        container::decode_downsized(&data, max_side)?
    } else {
        container::decode(&data)?
    };
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    imgio::save(output, &img)?;
    println!(
        "decoded {}x{} {} channels -> {} ({:.0} ms)",
        img.width, img.height, img.channels, output, ms
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return usage();
    }
    let result = match args[1].as_str() {
        "info" if args.len() == 3 => cmd_info(&args[2]),
        "encode" => cmd_encode(&args),
        "decode" => cmd_decode(&args),
        "-h" | "--help" | "help" => return usage(),
        other => Err(format!("unknown command: {}", other)),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::from(1)
        }
    }
}

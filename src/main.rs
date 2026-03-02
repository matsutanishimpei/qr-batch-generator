use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use qrcode::{QrCode, EcLevel};
use rusttype::{Font, Scale};
use std::fs;
use std::io::{BufRead, BufReader};

fn main() {
    println!("=== QRコード一括作成ツール ===");
    
    let input_file = "input.txt";
    let file = match fs::File::open(input_file) {
        Ok(f) => f,
        Err(_) => {
            fs::write(input_file, "https://example.com/001\n文字列サンプル123\n社員番号A-001\n").unwrap();
            println!("🚨 {} を新規作成しました。QR化したい文字列を追記して再度実行してください。", input_file);
            return;
        }
    };

    let reader = BufReader::new(file);
    let mut texts = Vec::new();
    for line in reader.lines() {
        if let Ok(l) = line {
            let t = l.trim().to_string();
            if !t.is_empty() {
                texts.push(t);
            }
        }
    }

    if texts.is_empty() {
        println!("🚨 {} に文字が入力されていません。", input_file);
        return;
    }

    fs::create_dir_all("output").unwrap();

    // システムの環境変数からWindowsフォルダを取得（環境依存の絶対パスを回避）
    let windir = std::env::var("windir").unwrap_or_else(|_| "C:\\Windows".to_string());
    let meiryo_path = format!("{}\\Fonts\\meiryo.ttc", windir);
    let gothic_path = format!("{}\\Fonts\\msgothic.ttc", windir);

    let font_data = fs::read(&meiryo_path).unwrap_or_else(|_| {
        fs::read(&gothic_path).expect("システムフォントが見つかりません。")
    });

    let font = Font::try_from_vec(font_data).expect("フォントの取得失敗");

    println!("生成を開始します...（合計 {} 件）\n", texts.len());

    let scale = Scale { x: 40.0, y: 40.0 };

    for (i, text) in texts.iter().enumerate() {
        let code = match QrCode::with_error_correction_level(text.as_bytes(), EcLevel::H) {
            Ok(c) => c,
            Err(e) => {
                println!("❌ [{}/{}] エラー: {} ({})", i + 1, texts.len(), text, e);
                continue;
            }
        };

        let mut img: RgbImage = code.render::<Rgb<u8>>()
            .min_dimensions(800, 800)
            .dark_color(Rgb([20, 30, 40]))
            .light_color(Rgb([255, 255, 255]))
            .build();

        let (img_width, img_height) = img.dimensions();

        let display_text = if text.chars().count() > 18 {
            let mut s: String = text.chars().take(17).collect();
            s.push_str("…");
            s
        } else {
            text.clone()
        };

        // テキストのサイズを計算（text_sizeの代わりにrusttypeの機能を使用）
        let v_metrics = font.v_metrics(scale);
        let height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;

        let width = font
            .layout(&display_text, scale, rusttype::point(0.0, v_metrics.ascent))
            .last()
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .unwrap_or(0.0).ceil() as u32;

        let padding_x: u32 = 24;
        let padding_y: u32 = 10;
        let bg_w = width + padding_x * 2;
        let bg_h = height + padding_y * 2;
        
        let bg_x = (img_width.saturating_sub(bg_w)) / 2;
        let bg_y = (img_height.saturating_sub(bg_h)) / 2;

        // 白枠を描画
        draw_filled_rect_mut(
            &mut img,
            Rect::at(bg_x as i32, bg_y as i32).of_size(bg_w, bg_h),
            Rgb([255, 255, 255])
        );

        let text_x = bg_x + padding_x;
        let text_y = bg_y + padding_y / 2;
        
        // draw_text_mutは u32 の座標を要求するため、型を合わせて描画
        draw_text_mut(
            &mut img,
            Rgb([20, 30, 40]),
            text_x,
            text_y,
            scale,
            &font,
            &display_text
        );

        let output_path = format!("output/qr_{:03}.png", i + 1);
        match img.save(&output_path) {
            Ok(_) => println!("✅ [{}/{}] 保存完了: {} (内容: {})", i + 1, texts.len(), output_path, display_text),
            Err(e) => println!("❌ [{}/{}] 保存失敗: {} ({})", i + 1, texts.len(), output_path, e),
        }
    }

    println!("\n🎉 すべて完了しました！ output フォルダをご確認ください！");
}

//! The interface fonts with and without the optional craft-fonts build input (`CRAFT_FONTS_DIR`).

use egui::epaint::text::{Fonts, TextOptions};
use egui::{Color32, FontFamily, FontId};
use pdfcraft_ui_egui::theme;

const JAPANESE: &str = "日本語の文字";
const CHINESE: &str = "简体中文欢迎";
const ARABIC: &str = "واحد اثنين";
const TELUGU: &str = "తెలుగు ఫైల్";

fn families() -> Vec<FontId> {
    vec![FontId::proportional(13.0), FontId::monospace(13.0), theme::medium(13.0), theme::semibold(17.0)]
}

/// Lays `text` out in every interface family and returns each galley's width.
fn layout_widths(fonts: &mut Fonts, text: &str) -> Vec<f32> {
    let mut view = fonts.with_pixels_per_point(2.0);
    families().into_iter().map(|id| view.layout_no_wrap(text.to_owned(), id, Color32::BLACK).size().x).collect()
}

/// Built with craft-fonts, Japanese text renders with real glyphs (no tofu) in every family,
/// from a craft-fonts face placed after the app's own fonts.
#[test]
fn japanese_ui_text_uses_craft_fonts() {
    if pdfcraft_fonts::ui_japanese_fonts().is_empty() {
        eprintln!("skipping japanese_ui_text_uses_craft_fonts: built without craft-fonts (set CRAFT_FONTS_DIR to run it)");
        return;
    }
    let defs = theme::font_definitions();
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let stack = &defs.families[&family];
        let first_jp = stack.iter().position(|n| n.starts_with("BIZ UDPGothic")).expect("BIZ UDPGothic is a fallback");
        let own = stack.iter().position(|n| n == "Inter" || n == "JetBrainsMono").expect("the app's own font");
        assert!(own < first_jp, "{family:?}: {stack:?}");
        assert!(stack[first_jp..].iter().all(|n| pdfcraft_fonts::CRAFT_FONTS.iter().any(|f| f.name() == *n)), "{stack:?}");
    }
    let mut fonts = Fonts::new(TextOptions::default(), defs);
    for id in families() {
        assert!(fonts.has_glyphs(&id, JAPANESE), "{id:?} lacks {JAPANESE}");
    }
    // Real glyphs are about a full em wide each; tofu boxes and missing glyphs are not.
    for w in layout_widths(&mut fonts, JAPANESE) {
        assert!(w > 13.0 * 0.8 * JAPANESE.chars().count() as f32, "{w}");
    }
}

/// Chinese mode orders the CJK fallback with the `Hans` group first, so one line never
/// mixes faces with different vertical metrics. Without an allowed `Hans` face in the
/// build input the order part still holds; glyph coverage follows the craft-fonts checkout.
#[test]
fn chinese_ui_text_prefers_the_chinese_face() {
    if pdfcraft_fonts::ui_chinese_fonts().is_empty() {
        eprintln!(
            "skipping chinese_ui_text_prefers_the_chinese_face: no Hans face bundled (set CRAFT_FONTS_DIR with an allowed Chinese face to run it)"
        );
        return;
    }
    let defs = theme::font_definitions_for(true);
    let hans: Vec<String> = pdfcraft_fonts::ui_chinese_fonts().iter().map(|f| f.name()).collect();
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let stack = &defs.families[&family];
        let first_zh = stack.iter().position(|n| hans.iter().any(|h| h == n)).expect("a Hans face is a fallback");
        let first_ja = stack.iter().position(|n| n.starts_with("BIZ UDPGothic")).expect("BIZ UDPGothic is a fallback");
        let own = stack.iter().position(|n| n == "Inter" || n == "JetBrainsMono").expect("the app's own font");
        assert!(own < first_zh && first_zh < first_ja, "{family:?}: {stack:?}");
    }
    let mut fonts = Fonts::new(TextOptions::default(), defs);
    for id in families() {
        assert!(fonts.has_glyphs(&id, CHINESE), "{id:?} lacks {CHINESE}");
    }
    for w in layout_widths(&mut fonts, CHINESE) {
        assert!(w > 13.0 * 0.8 * CHINESE.chars().count() as f32, "{w}");
    }
}

/// Built with a craft-fonts Arabic face, Arabic text has real glyphs in every family, from a
/// face placed after the app's own fonts.
#[test]
fn arabic_ui_text_uses_craft_fonts() {
    let arabic: Vec<String> = pdfcraft_fonts::ui_arabic_fonts().iter().map(|f| f.name()).collect();
    if arabic.is_empty() {
        eprintln!("skipping arabic_ui_text_uses_craft_fonts: no Arab face bundled (set CRAFT_FONTS_DIR with an Arabic face to run it)");
        return;
    }
    let defs = theme::font_definitions();
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let stack = &defs.families[&family];
        let first_ar = stack.iter().position(|n| arabic.contains(n)).expect("an Arab face is a fallback");
        let own = stack.iter().position(|n| n == "Inter" || n == "JetBrainsMono").expect("the app's own font");
        assert!(own < first_ar, "{family:?}: {stack:?}");
    }
    let mut fonts = Fonts::new(TextOptions::default(), defs);
    for id in families() {
        assert!(fonts.has_glyphs(&id, ARABIC), "{id:?} lacks {ARABIC}");
    }
}

/// Built with a craft-fonts Telugu face, Telugu text has real glyphs in every family, from a
/// face placed after the app's own fonts, and is shaped: the conjunct క్ష is narrower than క్
/// and ష held apart by a zero-width non-joiner.
#[test]
fn telugu_ui_text_uses_craft_fonts() {
    let telugu: Vec<String> = pdfcraft_fonts::CRAFT_FONTS.iter().filter(|f| f.covers("Telu")).map(|f| f.name()).collect();
    if telugu.is_empty() {
        eprintln!("skipping telugu_ui_text_uses_craft_fonts: no Telu face bundled (set CRAFT_FONTS_DIR with a Telugu face to run it)");
        return;
    }
    let defs = theme::font_definitions();
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let stack = &defs.families[&family];
        let first_te = stack.iter().position(|n| telugu.contains(n)).expect("a Telu face is a fallback");
        let own = stack.iter().position(|n| n == "Inter" || n == "JetBrainsMono").expect("the app's own font");
        assert!(own < first_te, "{family:?}: {stack:?}");
    }
    let mut fonts = Fonts::new(TextOptions::default(), defs);
    for id in families() {
        assert!(fonts.has_glyphs(&id, TELUGU), "{id:?} lacks {TELUGU}");
    }
    let joined = layout_widths(&mut fonts, "క్ష");
    let apart = layout_widths(&mut fonts, "క్\u{200c}ష");
    for (j, a) in joined.iter().zip(&apart) {
        assert!(j < a, "క్ష is one conjunct, narrower than క్‌ష: {j} vs {a}");
    }
}

/// On a machine with suitable installed fonts, the installed definitions put a last-resort face
/// for each uncovered script at the end of every family (so Arabic, and now Chinese, get real
/// glyphs); the embedded-only definitions never name those faces.
#[test]
fn system_fallback_fills_missing_scripts() {
    assert!(theme::font_definitions().font_data.keys().all(|k| !k.starts_with("system-")));
    let defs = theme::installed_font_definitions(false);
    let system: Vec<String> = defs.font_data.keys().filter(|k| k.starts_with("system-")).cloned().collect();
    if system.is_empty() {
        eprintln!("skipping system_fallback_fills_missing_scripts: no installed fallback font (or PDFCRAFT_SYSTEM_FONTS=0)");
        assert!(defs.families.values().all(|stack| stack.iter().all(|n| !n.starts_with("system-"))));
        return;
    }
    // Every family ends with the installed faces.
    for name in &system {
        for (family, stack) in &defs.families {
            assert!(stack.iter().any(|n| n == name), "{family:?} missing {name}");
        }
    }
    let mut fonts = Fonts::new(TextOptions::default(), defs);
    for id in families() {
        assert!(fonts.has_glyphs(&id, ARABIC), "{id:?} lacks {ARABIC}");
        assert!(fonts.has_glyphs(&id, "PdfCraft"), "{id:?}");
    }
    // A CJK fallback covers Chinese UI text (file names, Chinese locale strings).
    if system.iter().any(|n| n == "system-cjk") {
        let mut cjk_fonts = Fonts::new(TextOptions::default(), theme::installed_font_definitions(false));
        for id in families() {
            assert!(cjk_fonts.has_glyphs(&id, CHINESE), "{id:?} lacks {CHINESE}");
        }
    }
    assert!(layout_widths(&mut fonts, ARABIC).iter().all(|w| w.is_finite() && *w > 0.0));
}

/// Without craft-fonts the interface fonts still install and lay out any text (Japanese falls
/// back to egui's replacement glyph) without panicking; Latin text is unaffected.
#[test]
fn ui_fonts_work_without_craft_fonts() {
    let mut fonts = Fonts::new(TextOptions::default(), theme::font_definitions());
    for id in families() {
        assert!(fonts.has_glyphs(&id, "PdfCraft"), "{id:?}");
        assert_eq!(fonts.has_glyphs(&id, JAPANESE), !pdfcraft_fonts::ui_japanese_fonts().is_empty(), "{id:?}");
    }
    assert!(layout_widths(&mut fonts, JAPANESE).iter().all(|w| w.is_finite() && *w > 0.0));
    assert!(layout_widths(&mut fonts, "PdfCraft").iter().all(|w| *w > 20.0));
    let ctx = egui::Context::default();
    theme::install_fonts(&ctx);
}

/// Latin- and Cyrillic-script catalogs (Czech, Brazilian Portuguese, German, Spanish, French, Russian, Bulgarian, Hungarian, Ukrainian, Italian) are drawn entirely by the app's own faces
/// (Inter, JetBrains Mono): no letter falls through to egui's defaults or a CJK fallback. (egui's
/// `has_glyphs` can't answer this: with only the primary face it is also the replacement face.)
#[test]
fn primary_ui_fonts_cover_latin_and_cyrillic_catalogs() {
    use skrifa::MetadataProvider as _;
    let defs = theme::font_definitions();
    for (code, catalog) in [
        ("cs", include_str!("../src/i18n/cs.tsv")),
        ("pt-br", include_str!("../src/i18n/pt-br.tsv")),
        ("de", include_str!("../src/i18n/de.tsv")),
        ("es", include_str!("../src/i18n/es.tsv")),
        ("fr", include_str!("../src/i18n/fr.tsv")),
        ("ru", include_str!("../src/i18n/ru.tsv")),
        ("bg", include_str!("../src/i18n/bg.tsv")),
        ("hu", include_str!("../src/i18n/hu.tsv")),
        ("uk", include_str!("../src/i18n/uk.tsv")),
        ("it", include_str!("../src/i18n/it.tsv")),
    ] {
        let mut text = String::from("áčďéěíňóřšťúůýžÁČĎÉĚÍŇÓŘŠŤÚŮÝŽãõçâêôàÃÕÇÂÊÔÀñÑüÜäÄöÖőűß„“¿¡«»…ґҐєЄіІїЇ");
        for line in catalog.lines().filter(|l| !l.starts_with('#')) {
            if let Some(translation) = line.split('\t').nth(2) {
                text.extend(translation.chars().filter(|c| !c.is_whitespace()));
            }
        }
        for name in ["Inter", "Inter-Medium", "Inter-SemiBold", "JetBrainsMono"] {
            let data = &defs.font_data[name];
            let font = skrifa::FontRef::from_index(&data.font, data.index).unwrap();
            let charmap = font.charmap();
            let mut missing: Vec<char> = text.chars().filter(|c| charmap.map(*c).is_none()).collect();
            missing.sort_unstable();
            missing.dedup();
            assert!(missing.is_empty(), "{code}: {name} lacks {missing:?}");
        }
    }
}

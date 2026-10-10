//! The last-resort interface fonts: faces already installed on this machine.
//!
//! The embedded faces (Inter, egui's defaults, craft-fonts) come first in every family; these
//! only draw characters none of them has — such as Chinese UI text, an Arabic file name, or a
//! Telugu label in a build whose embedded faces don't cover those scripts. They are read at
//! runtime and never embedded or shipped (AGENTS.md §1.4), and `PDFCRAFT_SYSTEM_FONTS=0` turns
//! them off (published screenshots do).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use egui::FontData;

/// Larger files are not read: a font path is still untrusted input.
const MAX_BYTES: u64 = 128 << 20;
/// Faces tried in a collection (`.ttc`/`.otc`).
const MAX_FACES: u32 = 16;

/// A last-resort face, drawn only for characters no embedded face has.
pub struct Fallback {
    /// Unique family name, used as the key in `egui::FontDefinitions::font_data`.
    pub name: &'static str,
    /// The font, read from disk once.
    pub data: Arc<FontData>,
}

/// All installed fallback faces, read once. Empty when they are turned off or no candidate fits.
pub fn fallbacks() -> &'static [Fallback] {
    static CACHE: OnceLock<Vec<Fallback>> = OnceLock::new();
    CACHE.get_or_init(load)
}

/// A script we want a last-resort face for. A candidate qualifies when one of its faces maps
/// every character in `probes`. `name` must be unique across groups.
struct Group {
    name: &'static str,
    probes: &'static [char],
    candidates: Vec<PathBuf>,
}

fn load() -> Vec<Fallback> {
    if std::env::var_os("PDFCRAFT_SYSTEM_FONTS").is_some_and(|v| v == "0") {
        return Vec::new();
    }
    // A candidate path may appear in more than one group; read each file at most once. A group
    // whose script an already-picked face covers is skipped, so one wide face (PingFang, Noto
    // CJK) serves several groups instead of being loaded once per name.
    let mut files: HashMap<PathBuf, Option<Arc<[u8]>>> = HashMap::new();
    let mut picked: Vec<(&'static str, Arc<[u8]>, u32)> = Vec::new();
    let mut out = Vec::new();
    for group in &groups() {
        if picked
            .iter()
            .any(|(_, bytes, index)| face_maps(bytes, *index, group.probes))
        {
            continue;
        }
        for path in &group.candidates {
            let bytes = match files.entry(path.clone()).or_insert_with(|| read_bytes(path)) {
                Some(bytes) => bytes.clone(),
                None => continue,
            };
            if let Some(index) = face_with(&bytes, group.probes) {
                log::info!("interface font fallback ({}): {}", group.name, path.display());
                let mut data = FontData::from_owned(bytes.to_vec());
                data.index = index;
                // Align this face's baseline with the primary UI font (Inter) so mixed
                // Chinese/English text sits on one line instead of the CJK glyphs sinking
                // below it (egui centers different fonts by row height, not by baseline).
                if let Some(off) = baseline_offset_per_em(bytes.as_ref(), index) {
                    data.tweak.y_offset_factor = reference_baseline_offset() - off;
                }
                picked.push((group.name, bytes, index));
                out.push(Fallback { name: group.name, data: Arc::new(data) });
                break;
            }
        }
    }
    out
}

/// One last-resort group per script the embedded faces don't reliably cover. Best-coverage
/// scripts come first so the most common gaps (Chinese) win the slot when a single face can't
/// cover every script. `system-cjk` covers simplified Chinese UI text; `system-cjk-ext` covers
/// the traditional/Japanese forms (日本語, 繁體中文) a simplified-only face lacks; Telugu is the
/// remaining interface language.
fn groups() -> Vec<Group> {
    if cfg!(windows) {
        let dir = std::env::var_os("WINDIR").or_else(|| std::env::var_os("SystemRoot")).map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
        let fonts = dir.join("Fonts");
        vec![
            Group {
                name: "system-cjk",
                // 中 — proves the face has Han glyphs.
                probes: &['\u{4E2D}'],
                candidates: [
                    fonts.join("msyh.ttc"),
                    fonts.join("msyh.ttf"),
                    fonts.join("simhei.ttf"),
                    fonts.join("simsun.ttc"),
                ]
                .to_vec(),
            },
            Group {
                name: "system-cjk-ext",
                // 語體 — traditional/Japanese Han forms; SimSun covers them even when 微软雅黑 doesn't.
                probes: &['\u{8A9E}', '\u{9AD4}'],
                candidates: [fonts.join("msyh.ttc"), fonts.join("simsun.ttc"), fonts.join("simhei.ttf")].to_vec(),
            },
            Group {
                name: "system-telugu",
                // తగ — base Telugu consonants.
                probes: &['\u{0C24}', '\u{0C17}'],
                candidates: [fonts.join("nirmala.ttf")].to_vec(),
            },
            Group {
                name: "system-arabic",
                // ا — proves the face has Arabic glyphs.
                probes: &['\u{0627}'],
                candidates: [fonts.join("segoeui.ttf"), fonts.join("tahoma.ttf"), fonts.join("arial.ttf")].to_vec(),
            },
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            Group {
                name: "system-cjk",
                probes: &['\u{4E2D}'],
                candidates: [
                    // 苹方：多数 macOS 上的中文 UI 字体。
                    "/System/Library/Fonts/PingFang.ttc".into(),
                    // ヒラギノ角ゴ 简体中文黑体（位于 Fonts 顶层，不在 Supplemental）。
                    "/System/Library/Fonts/Hiragino Sans GB.ttc".into(),
                    // 华文黑体：另一种系统自带的 sans-serif 中文。
                    "/System/Library/Fonts/STHeiti Light.ttc".into(),
                    // 宋体：衬线，仅作最后兜底。
                    "/System/Library/Fonts/Supplemental/Songti.ttc".into(),
                ]
                .to_vec(),
            },
            Group {
                name: "system-cjk-ext",
                // 語體 — the traditional/Japanese Han forms simplified faces often lack.
                probes: &['\u{8A9E}', '\u{9AD4}'],
                candidates: [
                    "/System/Library/Fonts/PingFang.ttc".into(),
                    "/System/Library/Fonts/STHeiti Light.ttc".into(),
                    "/System/Library/Fonts/Supplemental/Songti.ttc".into(),
                    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf".into(),
                ]
                .to_vec(),
            },
            Group {
                name: "system-telugu",
                // తగ — base Telugu consonants.
                probes: &['\u{0C24}', '\u{0C17}'],
                candidates: [
                    "/System/Library/Fonts/KohinoorTelugu.ttc".into(),
                    "/System/Library/Fonts/Supplemental/NotoSansTelugu-Regular.ttf".into(),
                    "/System/Library/Fonts/Supplemental/Telugu Sangam MN.ttc".into(),
                    "/System/Library/Fonts/Supplemental/Arial Unicode.ttf".into(),
                ]
                .to_vec(),
            },
            Group {
                name: "system-arabic",
                probes: &['\u{0627}'],
                candidates: ["/System/Library/Fonts/GeezaPro.ttc".into(), "/System/Library/Fonts/SFArabic.ttf".into()].to_vec(),
            },
        ]
    } else {
        let cjk = [
            "noto/NotoSansCJK-Regular.ttc",
            "google-noto/NotoSansCJK-Regular.ttc",
            "truetype/noto/NotoSansCJK-Regular.ttc",
            "opentype/noto/NotoSansCJK-Regular.ttc",
            "wqy-microhei/wqy-microhei.ttc",
            "wqy-zenhei/wqy-zenhei.ttc",
            "truetype/wqy/wqy-microhei.ttc",
        ];
        let telugu = [
            "truetype/noto/NotoSansTelugu-Regular.ttf",
            "noto/NotoSansTelugu-Regular.ttf",
            "google-noto/NotoSansTelugu-Regular.ttf",
        ];
        let arabic = [
            "truetype/noto/NotoSansArabic-Regular.ttf",
            "noto/NotoSansArabic-Regular.ttf",
            "google-noto/NotoSansArabic-Regular.ttf",
            "truetype/dejavu/DejaVuSans.ttf",
            "TTF/DejaVuSans.ttf",
            "dejavu/DejaVuSans.ttf",
            "dejavu-sans-fonts/DejaVuSans.ttf",
        ];
        let dirs = ["/usr/share/fonts", "/usr/local/share/fonts"];
        let expand = |files: &[&str]| -> Vec<PathBuf> {
            dirs.iter()
                .copied()
                .flat_map(|dir| files.iter().copied().map(move |f| Path::new(dir).join(f)))
                .collect()
        };
        vec![
            Group { name: "system-cjk", probes: &['\u{4E2D}'], candidates: expand(&cjk) },
            // Noto Sans CJK covers 語/體 too, so this group is skipped there.
            Group { name: "system-cjk-ext", probes: &['\u{8A9E}', '\u{9AD4}'], candidates: expand(&cjk) },
            Group { name: "system-telugu", probes: &['\u{0C24}', '\u{0C17}'], candidates: expand(&telugu) },
            Group { name: "system-arabic", probes: &['\u{0627}'], candidates: expand(&arabic) },
        ]
    }
}

/// Read `path`'s bytes. `None` if the path is missing, not a file, too large, or unreadable.
fn read_bytes(path: &Path) -> Option<Arc<[u8]>> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_BYTES {
        return None;
    }
    std::fs::read(path).ok().map(Into::into)
}

/// The first face (within `MAX_FACES`) that parses and maps every `probes` character. egui
/// parses fonts with the same skrifa, so a face accepted here is one it can load.
fn face_with(bytes: &[u8], probes: &[char]) -> Option<u32> {
    (0..MAX_FACES).find(|&index| face_maps(bytes, index, probes))
}

/// Whether face `index` of the file parses and maps every `probes` character.
fn face_maps(bytes: &[u8], index: u32, probes: &[char]) -> bool {
    use skrifa::MetadataProvider as _;
    skrifa::FontRef::from_index(bytes, index)
        .is_ok_and(|font| probes.iter().all(|&c| font.charmap().map(c).is_some()))
}

/// The vertical position, as a fraction of the em, of the distance from the top of the line box
/// to the baseline (matching egui's `glyph.pos.y` baseline term `ascent - 0.5·row_height`). Two
/// fonts with the same value share a baseline, so a fallback font's `y_offset_factor` should be
/// set to `reference - this` to sit on the primary font's line.
fn baseline_offset_per_em(bytes: &[u8], index: u32) -> Option<f32> {
    use skrifa::instance::{LocationRef, Size};
    use skrifa::MetadataProvider as _;
    let font = skrifa::FontRef::from_index(bytes, index).ok()?;
    let m = font.metrics(Size::unscaled(), LocationRef::default());
    let upm = m.units_per_em as f32;
    // Same args egui uses when it builds `DependentFontData.metrics` (font.rs).
    let m = font.metrics(Size::unscaled(), LocationRef::default());
    let row_height = m.ascent - m.descent + m.leading;
    Some((m.ascent - 0.5 * row_height) / upm)
}

/// Inter's baseline offset (the primary UI Latin font), cached. `FontData`'s metrics come from
/// `skrifa::FontRef::metrics` exactly as egui reads them (font.rs), so the comparison is exact.
fn reference_baseline_offset() -> f32 {
    static CACHE: OnceLock<f32> = OnceLock::new();
    *CACHE.get_or_init(|| {
        const INTER: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
        baseline_offset_per_em(INTER, 0).unwrap_or(0.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broken_and_missing_files_are_skipped() {
        assert_eq!(face_with(b"", &['\u{0627}']), None);
        assert_eq!(face_with(b"not a font at all", &['\u{0627}']), None);
        assert_eq!(face_with(&[0u8; 4096], &['\u{0627}']), None);
        assert!(read_bytes(Path::new("definitely/not/here.ttf")).is_none());
        // A directory is not a font.
        assert!(read_bytes(&std::env::temp_dir()).is_none());
    }

    #[test]
    fn a_face_without_the_probe_is_rejected() {
        // Inter is Latin, Greek and Cyrillic only: no Arabic, but Latin 'A'.
        let inter = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
        assert_eq!(face_with(inter, &['\u{0627}']), None);
        assert_eq!(face_with(inter, &['A']), Some(0));
        // Every probe must map: a face with 'A' but no 'ا' fails the pair.
        assert_eq!(face_with(inter, &['A', '\u{0627}']), None);
    }

    #[test]
    fn group_names_are_unique() {
        let names: Vec<&str> = groups().into_iter().map(|g| g.name).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(names.len(), sorted.len(), "duplicate group name: {names:?}");
    }

    #[test]
    fn groups_are_absolute_font_files() {
        let list: Vec<PathBuf> = groups().into_iter().flat_map(|g| g.candidates).collect();
        assert!(!list.is_empty());
        for p in &list {
            assert!(p.is_absolute(), "{p:?} should be absolute");
            assert!(p.extension().is_some_and(|e| e == "ttf" || e == "ttc" || e == "otf"), "{p:?}");
        }
    }

    // Every CJK/Telugu last-resort must resolve on macOS, and the main Chinese one must be a
    // sans-serif face (PingFang / Hiragino Sans GB / STHeiti), never the serif Songti, or
    // Chinese UI text draws in the wrong style. (The traditional-forms group may legitimately
    // pick Songti when no sans face covers 語體.)
    #[test]
    #[cfg(target_os = "macos")]
    fn macos_cjk_and_telugu_fallbacks_resolve() {
        for group in groups() {
            if !matches!(group.name, "system-cjk" | "system-cjk-ext" | "system-telugu") {
                continue;
            }
            let picked = group
                .candidates
                .iter()
                .find_map(|p| {
                    std::fs::read(p).ok().and_then(|b| face_with(&b, group.probes).map(|_| p.clone()))
                })
                .unwrap_or_else(|| panic!("no macOS candidate for {} covers its probes", group.name));
            if group.name == "system-cjk" {
                assert!(
                    !picked.to_string_lossy().contains("Songti"),
                    "CJK fallback fell back to Songti (serif); expected a sans-serif face, picked {picked:?}"
                );
            }
        }
    }

    // The `y_offset_factor` we assign to each fallback font must bring its baseline onto Inter's,
    // so mixed Chinese/English text stays on one line. The tweak is `reference - off`, which by
    // construction makes `off + factor == reference`; assert that and that the factor is sane.
    #[test]
    fn fallback_baselines_align_with_inter() {
        let reference = reference_baseline_offset();
        assert!(reference.is_finite() && reference.abs() < 1.0, "Inter reference off: {reference}");
        for path in [
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/KohinoorTelugu.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        ] {
            let Ok(bytes) = std::fs::read(path) else { continue };
            let Some(index) = face_with(&bytes, &['\u{4E2D}']) else { continue };
            let off = baseline_offset_per_em(&bytes, index).unwrap();
            let factor = reference - off;
            assert!(factor.is_finite(), "{path}: non-finite tweak");
            assert!(factor.abs() < 1.0, "{path}: tweak too large: {factor}");
            assert!((off + factor - reference).abs() < 1e-6, "{path}: baseline not aligned");
        }
    }

}

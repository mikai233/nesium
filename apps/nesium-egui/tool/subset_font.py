import os
import re
import subprocess
import sys
from pathlib import Path

# Configuration
EGUI_ROOT = Path(__file__).parent.parent
SRC_DIR = EGUI_ROOT / "src"
# Using Flutter's fonts as source
FLUTTER_FONTS_DIR = EGUI_ROOT.parent / "nesium_flutter" / "assets" / "fonts"
SUBSET_OUT_DIR = EGUI_ROOT / "assets" / "fonts" / "subset"

ASSETS_FONTS = [
    "NotoSansSC-Regular.ttf",
    "NotoSansSC-Medium.ttf",
    "NotoSansSC-Bold.ttf",
]

# Essential characters that should always be included
DEFAULT_CHARS = (
    "abcdefghijklmnopqrstuvwxyz"
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    "0123456789"
    " .,!?:;-\" '()[]{}<>@#$%^&*+=~|\\/`"
    "™©®·…—"  # Common symbols
)

def extract_chars_from_file(file_path):
    chars = set()
    try:
        content = file_path.read_text(encoding="utf-8")
        # Match string literals in Rust: "..." or r"..." or r#"..."#
        # This is a bit simplified but usually captures most UI strings
        matches = re.findall(r'"([^"\\]*(?:\\.[^"\\]*)*)"', content)
        for m in matches:
            chars.update(m)
        
        # Match char literals: '...'
        char_matches = re.findall(r"'([^'\\]|\\.)'", content)
        for m in char_matches:
            # Handle escape sequences like \n, \t, etc.
            if len(m) == 1:
                chars.add(m)
            elif m.startswith('\\'):
                # We mainly care about the literal characters for CJK
                pass
    except Exception as e:
        print(f"Warning: Failed to read {file_path}: {e}")
    return chars

def main():
    if not SRC_DIR.exists():
        print(f"Error: {SRC_DIR} not found. Run from the project root (apps/nesium-egui).")
        sys.exit(1)

    print("Extracting characters from egui codebase...")
    all_chars = set(DEFAULT_CHARS)
    
    # Scan src directory
    for root, _, files in os.walk(SRC_DIR):
        for file in files:
            if file.endswith(".rs"):
                file_path = Path(root) / file
                all_chars.update(extract_chars_from_file(file_path))

    # Remove duplicates and sort
    text_to_subset = "".join(sorted(list(all_chars)))
    
    # Ensure output directory exists
    SUBSET_OUT_DIR.mkdir(parents=True, exist_ok=True)
    
    # Write characters to a temporary file for pyftsubset
    chars_file = EGUI_ROOT / "tool" / "used_chars.txt"
    chars_file.write_text(text_to_subset, encoding="utf-8")
    
    print(f"Total unique characters found: {len(text_to_subset)}")
    print(f"Characters saved to {chars_file}")

    for font_name in ASSETS_FONTS:
        font_path = FLUTTER_FONTS_DIR / font_name
        if not font_path.exists():
            print(f"Warning: Source font {font_path} not found, skipping.")
            continue

        output_path = SUBSET_OUT_DIR / font_name
        
        print(f"Subsetting {font_name}...")
        try:
            # Run pyftsubset via the module to avoid launcher issues on Windows
            # --text-file: include chars from file
            # --layout-features='*': keep kerning, etc.
            # --no-ignore-missing-glyphs: don't error if a char is missing
            subprocess.run([
                sys.executable,
                "-m",
                "fontTools.subset",
                str(font_path),
                f"--text-file={chars_file}",
                "--layout-features=*",
                f"--output-file={output_path}"
            ], check=True)
            
            orig_size = font_path.stat().st_size
            new_size = output_path.stat().st_size
            reduction = (orig_size - new_size) / orig_size * 100
            
            print(f"  Success: {orig_size / 1024 / 1024:.1f} MB -> {new_size / 1024:.1f} KB ({reduction:.1f}% reduction)")
            
        except subprocess.CalledProcessError as e:
            print(f"Error: pyftsubset failed for {font_name}: {e}")
        except Exception as e:
            print(f"Error: {e}")

    if chars_file.exists():
        chars_file.unlink()

if __name__ == "__main__":
    main()

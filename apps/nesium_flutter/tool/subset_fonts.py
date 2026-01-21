import os
import re
import subprocess
import sys
from pathlib import Path

# Configuration
FONTS_DIR = Path("assets/fonts")
LIB_DIR = Path("lib")
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
        # For .arb (JSON), extract all values
        if file_path.suffix == ".arb":
            # Simple match for everything inside quotes to be safe
            matches = re.findall(r'"([^"\\]*(?:\\.[^"\\]*)*)"', content)
            for m in matches:
                chars.update(m)
        # For .dart, extract string literals
        elif file_path.suffix == ".dart":
            # Match single quotes, double quotes, and triple quotes
            # Note: This is a bit simplified but usually captures most UI strings
            matches = re.findall(r"'(.*?)'|\"(.*?)\"", content, re.DOTALL)
            for m in matches:
                for group in m:
                    if group:
                        chars.update(group)
    except Exception as e:
        print(f"Warning: Failed to read {file_path}: {e}")
    return chars

def main():
    if not LIB_DIR.exists():
        print(f"Error: {LIB_DIR} not found. Run from the project root (apps/nesium_flutter).")
        sys.exit(1)

    print("Extracting characters from codebase...")
    all_chars = set(DEFAULT_CHARS)
    
    # Scan lib directory
    for root, _, files in os.walk(LIB_DIR):
        for file in files:
            if file.endswith(".dart") or file.endswith(".arb"):
                file_path = Path(root) / file
                all_chars.update(extract_chars_from_file(file_path))

    # Remove duplicates and sort
    text_to_subset = "".join(sorted(list(all_chars)))
    
    # Write characters to a temporary file for pyftsubset
    chars_file = Path("tool/used_chars.txt")
    chars_file.write_text(text_to_subset, encoding="utf-8")
    
    print(f"Total unique characters found: {len(text_to_subset)}")
    print(f"Characters saved to {chars_file}")

    for font_name in ASSETS_FONTS:
        font_path = FONTS_DIR / font_name
        if not font_path.exists():
            print(f"Warning: Font {font_path} not found, skipping.")
            continue

        output_path = font_path.with_suffix(".subset.ttf")
        
        print(f"Subsetting {font_name}...")
        try:
            # Run pyftsubset
            # --text-file: include chars from file
            # --layout-features='*': keep kerning, etc.
            # --glyph-names: keep names
            # --no-ignore-missing-glyphs: error if a char is missing (optional)
            subprocess.run([
                "pyftsubset",
                str(font_path),
                f"--text-file={chars_file}",
                "--layout-features=*",
                f"--output-file={output_path}"
            ], check=True)
            
            orig_size = font_path.stat().st_size
            new_size = output_path.stat().st_size
            reduction = (orig_size - new_size) / orig_size * 100
            
            print(f"  Success: {orig_size / 1024:.1f} KB -> {new_size / 1024:.1f} KB ({reduction:.1f}% reduction)")
            
            # Replace original with subset
            os.replace(output_path, font_path)
            
        except subprocess.CalledProcessError as e:
            print(f"Error: pyftsubset failed for {font_name}: {e}")
        except Exception as e:
            print(f"Error: {e}")

    if chars_file.exists():
        chars_file.unlink()

if __name__ == "__main__":
    main()

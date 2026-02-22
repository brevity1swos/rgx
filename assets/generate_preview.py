#!/usr/bin/env python3
"""Generate social preview image for rgx GitHub repository.
1280x640, Catppuccin Mocha theme, mock TUI with colored capture groups.
"""
from PIL import Image, ImageDraw, ImageFont

# Catppuccin Mocha palette
BASE = (30, 30, 46)
MANTLE = (24, 24, 37)
SURFACE0 = (49, 50, 68)
SURFACE1 = (69, 71, 90)
SURFACE2 = (88, 91, 112)
TEXT = (205, 214, 244)
SUBTEXT0 = (166, 173, 200)
SUBTEXT1 = (186, 194, 222)
LAVENDER = (180, 190, 254)
BLUE = (137, 180, 250)
SAPPHIRE = (116, 199, 236)
GREEN = (166, 227, 161)
YELLOW = (249, 226, 175)
PEACH = (250, 179, 135)
MAUVE = (203, 166, 247)
RED = (243, 139, 168)
PINK = (245, 194, 231)
TEAL = (148, 226, 213)
ROSEWATER = (245, 224, 220)


def blend(fg, bg, alpha):
    """Blend fg color onto bg color with alpha (0.0-1.0)."""
    return tuple(int(f * alpha + b * (1 - alpha)) for f, b in zip(fg, bg))


W, H = 1280, 640

img = Image.new("RGB", (W, H), BASE)
draw = ImageDraw.Draw(img)


# Fonts
def mono(size):
    return ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", size)


def sf(size):
    try:
        return ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", size)
    except Exception:
        return mono(size)


font_title = mono(52)
font_tagline = sf(22)
font_mono = mono(16)
font_label = mono(13)
font_small = mono(11)

# --- Left side: branding ---

# Title "rgx"
draw.text((60, 50), "rgx", fill=LAVENDER, font=font_title)

# Tagline
draw.text((60, 115), "regex101, but in your terminal", fill=SUBTEXT0, font=font_tagline)

# Feature pills
pills = [
    ("Real-time matching", GREEN),
    ("3 regex engines", BLUE),
    ("Capture groups", MAUVE),
    ("Plain-English explanations", YELLOW),
    ("Cross-platform", TEAL),
]

py = 170
for label, color in pills:
    tw = draw.textlength(label, font=font_small)
    pill_w = int(tw) + 20
    pill_h = 24
    bg = blend(color, BASE, 0.15)
    draw.rounded_rectangle(
        [(60, py), (60 + pill_w, py + pill_h)],
        radius=12,
        fill=bg,
        outline=blend(color, BASE, 0.4),
        width=1,
    )
    draw.text((70, py + 5), label, fill=color, font=font_small)
    py += 32

# Install command
py += 10
draw.text((60, py), "$ cargo install rgx-cli", fill=SURFACE2, font=mono(15))

# --- Right side: mock TUI ---

# TUI window frame
tx, ty = 480, 40
tw, th = 760, 560
# Window background
draw.rounded_rectangle([(tx, ty), (tx + tw, ty + th)], radius=10, fill=MANTLE)
# Title bar
draw.rounded_rectangle(
    [(tx, ty), (tx + tw, ty + 30)],
    radius=10,
    fill=SURFACE0,
)
# Square off bottom corners of title bar
draw.rectangle([(tx, ty + 20), (tx + tw, ty + 30)], fill=SURFACE0)
# Window dots
for i, c in enumerate([RED, YELLOW, GREEN]):
    draw.ellipse(
        [(tx + 12 + i * 22, ty + 8), (tx + 26 + i * 22, ty + 22)],
        fill=c,
    )
# Title bar text
draw.text((tx + 320, ty + 7), "rgx", fill=SUBTEXT0, font=font_label)

# Content area starts
cx, cy = tx + 16, ty + 42

# --- Pattern input ---
bbox = draw.textbbox((cx, cy), " Pattern ", font=font_label)
draw.rectangle(bbox, fill=BLUE)
draw.text((cx, cy), " Pattern ", fill=MANTLE, font=font_label)

# Pattern text with syntax coloring
px = cx + 100
pattern_parts = [
    ("(", MAUVE),
    ("\\w+", TEXT),
    (")", MAUVE),
    ("@", SURFACE2),
    ("(", PEACH),
    ("\\w+", TEXT),
    (")", PEACH),
    ("\\.", SURFACE2),
    ("(", GREEN),
    ("\\w+", TEXT),
    (")", GREEN),
]
for text, color in pattern_parts:
    draw.text((px, cy), text, fill=color, font=font_mono)
    px += draw.textlength(text, font=font_mono)

# Cursor
draw.rectangle([(px, cy), (px + 10, cy + 17)], fill=LAVENDER)

cy += 30

# --- Test string input ---
bbox = draw.textbbox((cx, cy), " Test String ", font=font_label)
draw.rectangle(bbox, fill=GREEN)
draw.text((cx, cy), " Test String ", fill=MANTLE, font=font_label)

# Test string with match highlighting
tsx = cx + 130
test_parts = [
    ("user", MAUVE, True),
    ("@", TEXT, False),
    ("example", PEACH, True),
    (".", TEXT, False),
    ("com", GREEN, True),
    ("  ", TEXT, False),
    ("admin", MAUVE, True),
    ("@", TEXT, False),
    ("test", PEACH, True),
    (".", TEXT, False),
    ("org", GREEN, True),
]
for text, color, highlight in test_parts:
    tw_t = draw.textlength(text, font=font_mono)
    if highlight:
        bg = blend(color, MANTLE, 0.2)
        draw.rectangle(
            [(tsx, cy + 1), (tsx + tw_t, cy + 18)],
            fill=bg,
        )
        draw.text((tsx, cy), text, fill=color, font=font_mono)
    else:
        draw.text((tsx, cy), text, fill=SUBTEXT0, font=font_mono)
    tsx += tw_t

cy += 36

# --- Separator ---
draw.line([(cx, cy), (cx + tw - 32, cy)], fill=SURFACE0, width=1)
cy += 8

# --- Match results ---
bbox = draw.textbbox((cx, cy), " Matches (2) ", font=font_label)
draw.rectangle(bbox, fill=MAUVE)
draw.text((cx, cy), " Matches (2) ", fill=MANTLE, font=font_label)
cy += 22

# Match 1
draw.text((cx, cy), "Match 1:", fill=SUBTEXT0, font=font_small)
draw.text((cx + 80, cy), "user@example.com", fill=TEXT, font=font_small)
cy += 18
for label, val, color in [
    ("  Group 1:", "user", MAUVE),
    ("  Group 2:", "example", PEACH),
    ("  Group 3:", "com", GREEN),
]:
    draw.text((cx, cy), label, fill=SURFACE2, font=font_small)
    draw.text((cx + 90, cy), val, fill=color, font=font_small)
    cy += 16

cy += 4
draw.text((cx, cy), "Match 2:", fill=SUBTEXT0, font=font_small)
draw.text((cx + 80, cy), "admin@test.org", fill=TEXT, font=font_small)
cy += 18
for label, val, color in [
    ("  Group 1:", "admin", MAUVE),
    ("  Group 2:", "test", PEACH),
    ("  Group 3:", "org", GREEN),
]:
    draw.text((cx, cy), label, fill=SURFACE2, font=font_small)
    draw.text((cx + 90, cy), val, fill=color, font=font_small)
    cy += 16

cy += 10

# --- Separator ---
draw.line([(cx, cy), (cx + tw - 32, cy)], fill=SURFACE0, width=1)
cy += 8

# --- Explanation panel ---
bbox = draw.textbbox((cx, cy), " Explanation ", font=font_label)
draw.rectangle(bbox, fill=YELLOW)
draw.text((cx, cy), " Explanation ", fill=MANTLE, font=font_label)
cy += 24

explanations = [
    ("(\\w+)", "Group 1: one or more word characters", MAUVE),
    ("@", "literal '@'", SUBTEXT0),
    ("(\\w+)", "Group 2: one or more word characters", PEACH),
    ("\\.", "literal '.'", SUBTEXT0),
    ("(\\w+)", "Group 3: one or more word characters", GREEN),
]
for pattern, desc, color in explanations:
    pw = draw.textlength(pattern, font=font_small)
    draw.text((cx + 8, cy), pattern, fill=color, font=font_small)
    draw.text((cx + 8 + pw + 10, cy), "\u2192", fill=SURFACE2, font=font_small)
    draw.text((cx + 8 + pw + 30, cy), desc, fill=SUBTEXT1, font=font_small)
    cy += 18

cy += 10

# --- Separator ---
draw.line([(cx, cy), (cx + tw - 32, cy)], fill=SURFACE0, width=1)
cy += 8

# --- Status bar ---
draw.rounded_rectangle([(cx, cy), (cx + 90, cy + 20)], radius=4, fill=SURFACE0)
draw.text((cx + 6, cy + 3), "Rust regex", fill=BLUE, font=font_small)

fx = cx + 100
draw.text((fx, cy + 3), "Flags: i  m  s  u  x", fill=SURFACE2, font=font_small)

draw.text((cx + 400, cy + 3), "2 matches  \u2022  3 groups", fill=SUBTEXT0, font=font_small)

# Save
out_path = "assets/social-preview.png"
img.save(out_path, "PNG")
print(f"Saved {out_path} ({W}x{H})")

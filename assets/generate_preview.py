#!/usr/bin/env python3
"""Generate social preview image for rgx GitHub repository.
1280x640, Catppuccin Mocha theme, mock TUI with colored capture groups and replace preview.
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
    ("Replace/substitution", PEACH),
    ("Undo/redo + history", YELLOW),
    ("Mouse support", TEAL),
    ("Clipboard copy", PINK),
]

py = 165
for label, color in pills:
    tw = draw.textlength(label, font=font_small)
    pill_w = int(tw) + 20
    pill_h = 22
    bg = blend(color, BASE, 0.15)
    draw.rounded_rectangle(
        [(60, py), (60 + pill_w, py + pill_h)],
        radius=11,
        fill=bg,
        outline=blend(color, BASE, 0.4),
        width=1,
    )
    draw.text((70, py + 4), label, fill=color, font=font_small)
    py += 28

# Install command
py += 8
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

cy += 28

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

cy += 28

# --- Replace input ---
bbox = draw.textbbox((cx, cy), " Replacement ", font=font_label)
draw.rectangle(bbox, fill=PEACH)
draw.text((cx, cy), " Replacement ", fill=MANTLE, font=font_label)

rpx = cx + 130
draw.text((rpx, cy), "$1 AT $2", fill=TEXT, font=font_mono)

cy += 28

# --- Separator ---
draw.line([(cx, cy), (cx + tw - 32, cy)], fill=SURFACE0, width=1)
cy += 6

# --- Match results (left half) and Explanation (right half) ---
# Split the remaining area into two columns
left_x = cx
right_x = cx + (tw - 32) // 2 + 10
col_w = (tw - 32) // 2 - 10

# --- Left column: Matches + Replace Preview ---
my = cy
bbox = draw.textbbox((left_x, my), " Matches (2) | Preview ", font=font_label)
draw.rectangle(bbox, fill=MAUVE)
draw.text((left_x, my), " Matches (2) | Preview ", fill=MANTLE, font=font_label)
my += 20

# Match 1 — selected (with >> prefix)
draw.text((left_x, my), ">>", fill=BLUE, font=font_small)
# Selection highlight background
sel_w = col_w - 4
draw.rectangle(
    [(left_x + 20, my), (left_x + sel_w, my + 14)],
    fill=SURFACE1,
)
draw.text((left_x + 22, my), "Match 1 [0-16]:", fill=BLUE, font=font_small)
draw.text((left_x + 160, my), '"user@example.com"', fill=GREEN, font=font_small)
my += 16
for label, val, color in [
    ("  Group 1:", "user", MAUVE),
    ("  Group 2:", "example", PEACH),
    ("  Group 3:", "com", GREEN),
]:
    draw.text((left_x + 8, my), label, fill=SURFACE2, font=font_small)
    draw.text((left_x + 90, my), val, fill=color, font=font_small)
    my += 14

my += 2
draw.text((left_x + 8, my), "Match 2 [17-31]:", fill=BLUE, font=font_small)
draw.text((left_x + 160, my), '"admin@test.org"', fill=GREEN, font=font_small)
my += 16
for label, val, color in [
    ("  Group 1:", "admin", MAUVE),
    ("  Group 2:", "test", PEACH),
    ("  Group 3:", "org", GREEN),
]:
    draw.text((left_x + 8, my), label, fill=SURFACE2, font=font_small)
    draw.text((left_x + 90, my), val, fill=color, font=font_small)
    my += 14

# Replace preview separator
my += 6
draw.text(
    (left_x + 8, my), "\u2500\u2500\u2500 Replace Preview \u2500\u2500\u2500",
    fill=SURFACE2, font=font_small,
)
my += 16

# Replace preview: "user AT example admin AT test"
# Replaced parts get green-on-dark highlight
preview_parts = [
    ("user AT example", True),
    (".com ", False),
    ("admin AT test", True),
    (".org", False),
]
rpx = left_x + 8
for text, is_replaced in preview_parts:
    tw_r = draw.textlength(text, font=font_small)
    if is_replaced:
        draw.rectangle(
            [(rpx, my), (rpx + tw_r, my + 14)],
            fill=blend(GREEN, MANTLE, 0.25),
        )
        draw.text((rpx, my), text, fill=GREEN, font=font_small)
    else:
        draw.text((rpx, my), text, fill=SUBTEXT0, font=font_small)
    rpx += tw_r

# --- Right column: Explanation ---
ey = cy
bbox = draw.textbbox((right_x, ey), " Explanation ", font=font_label)
draw.rectangle(bbox, fill=YELLOW)
draw.text((right_x, ey), " Explanation ", fill=MANTLE, font=font_label)
ey += 22

explanations = [
    ("(\\w+)", "Group 1: one or more word chars", MAUVE),
    ("@", "literal '@'", SUBTEXT0),
    ("(\\w+)", "Group 2: one or more word chars", PEACH),
    ("\\.", "literal '.'", SUBTEXT0),
    ("(\\w+)", "Group 3: one or more word chars", GREEN),
]
for pattern, desc, color in explanations:
    pw = draw.textlength(pattern, font=font_small)
    draw.text((right_x + 8, ey), pattern, fill=color, font=font_small)
    draw.text((right_x + 8 + pw + 8, ey), "\u2192", fill=SURFACE2, font=font_small)
    draw.text((right_x + 8 + pw + 24, ey), desc, fill=SUBTEXT1, font=font_small)
    ey += 18

# --- Status bar at bottom ---
sy = ty + th - 24
draw.rectangle([(tx + 2, sy), (tx + tw - 2, sy + 18)], fill=SURFACE0)

# Engine badge
draw.rounded_rectangle([(tx + 8, sy + 1), (tx + 98, sy + 17)], radius=3, fill=BLUE)
draw.text((tx + 14, sy + 3), "Rust regex", fill=MANTLE, font=font_small)

# Flag indicators
fx = tx + 106
flags = [("i", False), ("m", False), ("s", False), ("u", False), ("x", False)]
for name, active in flags:
    if active:
        draw.rounded_rectangle([(fx, sy + 1), (fx + 18, sy + 17)], radius=3, fill=GREEN)
        draw.text((fx + 4, sy + 3), name, fill=MANTLE, font=font_small)
    else:
        draw.text((fx + 4, sy + 3), name, fill=SURFACE2, font=font_small)
    fx += 22

# Hint text
draw.text(
    (fx + 10, sy + 3),
    "Tab: switch | Ctrl+E: engine | Ctrl+Z: undo | Ctrl+Y: copy | F1: help",
    fill=SUBTEXT0, font=font_small,
)

# Save
out_path = "assets/social-preview.png"
img.save(out_path, "PNG")
print(f"Saved {out_path} ({W}x{H})")

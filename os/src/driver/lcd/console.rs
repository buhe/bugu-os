use core::fmt;

use super::lcd_colors::rgb565;
use super::coord::Coord;
use super::palette_xterm256::PALETTE;

/** Display width in pixels */
pub const DISP_WIDTH: u16 = 320;
/** Display height in pixels */
pub const DISP_HEIGHT: u16 = 240;
/** Number of pixels in display */
pub const DISP_PIXELS: usize = (DISP_WIDTH as usize) * (DISP_HEIGHT as usize);
const GRID_WIDTH: u16 = DISP_WIDTH / 8;
const GRID_HEIGHT: u16 = DISP_HEIGHT / 8;
const GRID_CELLS: usize = (GRID_WIDTH as usize) * (GRID_HEIGHT as usize);
const DEF_FG: u16 = rgb565(192, 192, 192);
const DEF_BG: u16 = rgb565(0, 0, 0);

pub type ScreenImage = [u32; DISP_PIXELS / 2];

pub use super::color::Color;

/** Cell flags. */
#[allow(non_snake_case)]
pub mod CellFlags {
    /** Cell contains a color font character if this flag is set, and `ch` is an offset into
     * the color font. If not set, `ch` is an offset into the normal b/w bitmap font.
     */
    pub const COLOR: u16 = 1;
}

/** One character cell */
#[derive(Copy, Clone)]
pub struct Cell {
    /** Foreground color in RGB565 */
    fg: u16,
    /** Background color in RGB565 */
    bg: u16,
    /** Font index. The only hard requirement on the font is that 0 is an empty glyph. */
    ch: u16,
    /** Cell flags (see CellFlags) */
    flags: u16,
}

enum State {
    Initial,
    Escape,
    CSI,
    Xterm,
}

enum Sgr {
    Initial,
    SpecialFg,
    SpecialBg,
    Fg256,
    Bg256,
    FgR,
    BgR,
    FgG,
    BgG,
    FgB,
    BgB,
}

/** Visual attributes of console */
pub struct Console {
    /** Map unicode character to font index and flags word. */
    // map_utf: &'static dyn Fn(char) -> (u16, u16),
    /** Standard font */
    pub font: &'static [[u8; 8]],
    /** Color font */
    pub color_font: &'static [[u32; 32]],
    /** Dirty flag */
    pub dirty: bool,
    /** Array of character cells representing console */
    cells: [Cell; GRID_CELLS],
    /** Cursor position */
    cursor_pos: Coord,
    /** Cursor visible flag */
    cursor_visible: bool,
    /** Default foreground */
    def_fg: u16,
    /** Default background */
    def_bg: u16,
    /** Current foreground */
    cur_fg: u16,
    /** Current background */
    cur_bg: u16,
    /** Current escape state */
    state: State,
    /** Current CSI parameter */
    idx: usize,
    /** CSI parameters */
    num: [u16; 16],
}

impl Console {
    /** Create new, empty console */
    pub fn new(font: &'static [[u8; 8]], color_font: Option<&'static [[u32; 32]]>) -> Console {
        Console {
            font,
            color_font: color_font.unwrap_or(&[]),
            dirty: false,
            cells: [Cell {
                fg: DEF_FG,
                bg: DEF_BG,
                ch: 0,
                flags: 0,
            }; GRID_CELLS],
            cursor_pos: Coord::new(0, 0),
            cursor_visible: true,
            def_fg: DEF_FG,
            def_bg: DEF_BG,
            cur_fg: DEF_FG,
            cur_bg: DEF_BG,
            state: State::Initial,
            idx: 0,
            num: [0; 16],
        }
    }

    fn map_utf(&self, ch: char) -> (u16, u16) {
    (match ch {
        '\u{0000}' => 0x00, // NUL
        '\u{263a}' => 0x01, // WHITE SMILING FACE
        '\u{263b}' => 0x02, // BLACK SMILING FACE
        '\u{2665}' => 0x03, // BLACK HEART SUIT
        '\u{2666}' => 0x04, // BLACK DIAMOND SUIT
        '\u{2663}' => 0x05, // BLACK CLUB SUIT
        '\u{2660}' => 0x06, // BLACK SPADE SUIT
        '\u{2022}' => 0x07, // BULLET
        '\u{25d8}' => 0x08, // INVERSE BULLET
        '\u{25cb}' => 0x09, // WHITE CIRCLE
        '\u{25d9}' => 0x0a, // INVERSE WHITE CIRCLE
        '\u{2642}' => 0x0b, // MALE SIGN
        '\u{2640}' => 0x0c, // FEMALE SIGN
        '\u{266a}' => 0x0d, // EIGHTH NOTE
        '\u{266b}' => 0x0e, // BEAMED EIGHTH NOTES
        '\u{263c}' => 0x0f, // WHITE SUN WITH RAYS
        '\u{25ba}' => 0x10, // BLACK RIGHT-POINTING POINTER
        '\u{25c4}' => 0x11, // BLACK LEFT-POINTING POINTER
        '\u{2195}' => 0x12, // UP DOWN ARROW
        '\u{203c}' => 0x13, // DOUBLE EXCLAMATION MARK
        '\u{00b6}' => 0x14, // PILCROW SIGN
        '\u{00a7}' => 0x15, // SECTION SIGN
        '\u{25ac}' => 0x16, // BLACK RECTANGLE
        '\u{21a8}' => 0x17, // UP DOWN ARROW WITH BASE
        '\u{2191}' => 0x18, // UPWARDS ARROW
        '\u{2193}' => 0x19, // DOWNWARDS ARROW
        '\u{2192}' => 0x1a, // RIGHTWARDS ARROW
        '\u{2190}' => 0x1b, // LEFTWARDS ARROW
        '\u{221f}' => 0x1c, // RIGHT ANGLE
        '\u{2194}' => 0x1d, // LEFT RIGHT ARROW
        '\u{25b2}' => 0x1e, // BLACK UP-POINTING TRIANGLE
        '\u{25bc}' => 0x1f, // BLACK DOWN-POINTING TRIANGLE
        '\u{0020}' => 0x20, // SPACE
        '\u{0021}' => 0x21, // EXCLAMATION MARK
        '\u{0022}' => 0x22, // QUOTATION MARK
        '\u{0023}' => 0x23, // NUMBER SIGN
        '\u{0024}' => 0x24, // DOLLAR SIGN
        '\u{0025}' => 0x25, // PERCENT SIGN
        '\u{0026}' => 0x26, // AMPERSAND
        '\u{0027}' => 0x27, // APOSTROPHE
        '\u{0028}' => 0x28, // LEFT PARENTHESIS
        '\u{0029}' => 0x29, // RIGHT PARENTHESIS
        '\u{002a}' => 0x2a, // ASTERISK
        '\u{002b}' => 0x2b, // PLUS SIGN
        '\u{002c}' => 0x2c, // COMMA
        '\u{002d}' => 0x2d, // HYPHEN-MINUS
        '\u{002e}' => 0x2e, // FULL STOP
        '\u{002f}' => 0x2f, // SOLIDUS
        '\u{0030}' => 0x30, // DIGIT ZERO
        '\u{0031}' => 0x31, // DIGIT ONE
        '\u{0032}' => 0x32, // DIGIT TWO
        '\u{0033}' => 0x33, // DIGIT THREE
        '\u{0034}' => 0x34, // DIGIT FOUR
        '\u{0035}' => 0x35, // DIGIT FIVE
        '\u{0036}' => 0x36, // DIGIT SIX
        '\u{0037}' => 0x37, // DIGIT SEVEN
        '\u{0038}' => 0x38, // DIGIT EIGHT
        '\u{0039}' => 0x39, // DIGIT NINE
        '\u{003a}' => 0x3a, // COLON
        '\u{003b}' => 0x3b, // SEMICOLON
        '\u{003c}' => 0x3c, // LESS-THAN SIGN
        '\u{003d}' => 0x3d, // EQUALS SIGN
        '\u{003e}' => 0x3e, // GREATER-THAN SIGN
        '\u{003f}' => 0x3f, // QUESTION MARK
        '\u{0040}' => 0x40, // COMMERCIAL AT
        '\u{0041}' => 0x41, // LATIN CAPITAL LETTER A
        '\u{0042}' => 0x42, // LATIN CAPITAL LETTER B
        '\u{0043}' => 0x43, // LATIN CAPITAL LETTER C
        '\u{0044}' => 0x44, // LATIN CAPITAL LETTER D
        '\u{0045}' => 0x45, // LATIN CAPITAL LETTER E
        '\u{0046}' => 0x46, // LATIN CAPITAL LETTER F
        '\u{0047}' => 0x47, // LATIN CAPITAL LETTER G
        '\u{0048}' => 0x48, // LATIN CAPITAL LETTER H
        '\u{0049}' => 0x49, // LATIN CAPITAL LETTER I
        '\u{004a}' => 0x4a, // LATIN CAPITAL LETTER J
        '\u{004b}' => 0x4b, // LATIN CAPITAL LETTER K
        '\u{004c}' => 0x4c, // LATIN CAPITAL LETTER L
        '\u{004d}' => 0x4d, // LATIN CAPITAL LETTER M
        '\u{004e}' => 0x4e, // LATIN CAPITAL LETTER N
        '\u{004f}' => 0x4f, // LATIN CAPITAL LETTER O
        '\u{0050}' => 0x50, // LATIN CAPITAL LETTER P
        '\u{0051}' => 0x51, // LATIN CAPITAL LETTER Q
        '\u{0052}' => 0x52, // LATIN CAPITAL LETTER R
        '\u{0053}' => 0x53, // LATIN CAPITAL LETTER S
        '\u{0054}' => 0x54, // LATIN CAPITAL LETTER T
        '\u{0055}' => 0x55, // LATIN CAPITAL LETTER U
        '\u{0056}' => 0x56, // LATIN CAPITAL LETTER V
        '\u{0057}' => 0x57, // LATIN CAPITAL LETTER W
        '\u{0058}' => 0x58, // LATIN CAPITAL LETTER X
        '\u{0059}' => 0x59, // LATIN CAPITAL LETTER Y
        '\u{005a}' => 0x5a, // LATIN CAPITAL LETTER Z
        '\u{005b}' => 0x5b, // LEFT SQUARE BRACKET
        '\u{005c}' => 0x5c, // REVERSE SOLIDUS
        '\u{005d}' => 0x5d, // RIGHT SQUARE BRACKET
        '\u{005e}' => 0x5e, // CIRCUMFLEX ACCENT
        '\u{005f}' => 0x5f, // LOW LINE
        '\u{0060}' => 0x60, // GRAVE ACCENT
        '\u{0061}' => 0x61, // LATIN SMALL LETTER A
        '\u{0062}' => 0x62, // LATIN SMALL LETTER B
        '\u{0063}' => 0x63, // LATIN SMALL LETTER C
        '\u{0064}' => 0x64, // LATIN SMALL LETTER D
        '\u{0065}' => 0x65, // LATIN SMALL LETTER E
        '\u{0066}' => 0x66, // LATIN SMALL LETTER F
        '\u{0067}' => 0x67, // LATIN SMALL LETTER G
        '\u{0068}' => 0x68, // LATIN SMALL LETTER H
        '\u{0069}' => 0x69, // LATIN SMALL LETTER I
        '\u{006a}' => 0x6a, // LATIN SMALL LETTER J
        '\u{006b}' => 0x6b, // LATIN SMALL LETTER K
        '\u{006c}' => 0x6c, // LATIN SMALL LETTER L
        '\u{006d}' => 0x6d, // LATIN SMALL LETTER M
        '\u{006e}' => 0x6e, // LATIN SMALL LETTER N
        '\u{006f}' => 0x6f, // LATIN SMALL LETTER O
        '\u{0070}' => 0x70, // LATIN SMALL LETTER P
        '\u{0071}' => 0x71, // LATIN SMALL LETTER Q
        '\u{0072}' => 0x72, // LATIN SMALL LETTER R
        '\u{0073}' => 0x73, // LATIN SMALL LETTER S
        '\u{0074}' => 0x74, // LATIN SMALL LETTER T
        '\u{0075}' => 0x75, // LATIN SMALL LETTER U
        '\u{0076}' => 0x76, // LATIN SMALL LETTER V
        '\u{0077}' => 0x77, // LATIN SMALL LETTER W
        '\u{0078}' => 0x78, // LATIN SMALL LETTER X
        '\u{0079}' => 0x79, // LATIN SMALL LETTER Y
        '\u{007a}' => 0x7a, // LATIN SMALL LETTER Z
        '\u{007b}' => 0x7b, // LEFT CURLY BRACKET
        '\u{007c}' => 0x7c, // VERTICAL LINE
        '\u{007d}' => 0x7d, // RIGHT CURLY BRACKET
        '\u{007e}' => 0x7e, // TILDE
        '\u{2302}' => 0x7f, // HOUSE
        '\u{00c7}' => 0x80, // LATIN CAPITAL LETTER C WITH CEDILLA
        '\u{00fc}' => 0x81, // LATIN SMALL LETTER U WITH DIAERESIS
        '\u{00e9}' => 0x82, // LATIN SMALL LETTER E WITH ACUTE
        '\u{00e2}' => 0x83, // LATIN SMALL LETTER A WITH CIRCUMFLEX
        '\u{00e4}' => 0x84, // LATIN SMALL LETTER A WITH DIAERESIS
        '\u{00e0}' => 0x85, // LATIN SMALL LETTER A WITH GRAVE
        '\u{00e5}' => 0x86, // LATIN SMALL LETTER A WITH RING ABOVE
        '\u{00e7}' => 0x87, // LATIN SMALL LETTER C WITH CEDILLA
        '\u{00ea}' => 0x88, // LATIN SMALL LETTER E WITH CIRCUMFLEX
        '\u{00eb}' => 0x89, // LATIN SMALL LETTER E WITH DIAERESIS
        '\u{00e8}' => 0x8a, // LATIN SMALL LETTER E WITH GRAVE
        '\u{00ef}' => 0x8b, // LATIN SMALL LETTER I WITH DIAERESIS
        '\u{00ee}' => 0x8c, // LATIN SMALL LETTER I WITH CIRCUMFLEX
        '\u{00ec}' => 0x8d, // LATIN SMALL LETTER I WITH GRAVE
        '\u{00c4}' => 0x8e, // LATIN CAPITAL LETTER A WITH DIAERESIS
        '\u{00c5}' => 0x8f, // LATIN CAPITAL LETTER A WITH RING ABOVE
        '\u{00c9}' => 0x90, // LATIN CAPITAL LETTER E WITH ACUTE
        '\u{00e6}' => 0x91, // LATIN SMALL LETTER AE
        '\u{00c6}' => 0x92, // LATIN CAPITAL LETTER AE
        '\u{00f4}' => 0x93, // LATIN SMALL LETTER O WITH CIRCUMFLEX
        '\u{00f6}' => 0x94, // LATIN SMALL LETTER O WITH DIAERESIS
        '\u{00f2}' => 0x95, // LATIN SMALL LETTER O WITH GRAVE
        '\u{00fb}' => 0x96, // LATIN SMALL LETTER U WITH CIRCUMFLEX
        '\u{00f9}' => 0x97, // LATIN SMALL LETTER U WITH GRAVE
        '\u{00ff}' => 0x98, // LATIN SMALL LETTER Y WITH DIAERESIS
        '\u{00d6}' => 0x99, // LATIN CAPITAL LETTER O WITH DIAERESIS
        '\u{00dc}' => 0x9a, // LATIN CAPITAL LETTER U WITH DIAERESIS
        '\u{00a2}' => 0x9b, // CENT SIGN
        '\u{00a3}' => 0x9c, // POUND SIGN
        '\u{00a5}' => 0x9d, // YEN SIGN
        '\u{20a7}' => 0x9e, // PESETA SIGN
        '\u{0192}' => 0x9f, // LATIN SMALL LETTER F WITH HOOK
        '\u{00e1}' => 0xa0, // LATIN SMALL LETTER A WITH ACUTE
        '\u{00ed}' => 0xa1, // LATIN SMALL LETTER I WITH ACUTE
        '\u{00f3}' => 0xa2, // LATIN SMALL LETTER O WITH ACUTE
        '\u{00fa}' => 0xa3, // LATIN SMALL LETTER U WITH ACUTE
        '\u{00f1}' => 0xa4, // LATIN SMALL LETTER N WITH TILDE
        '\u{00d1}' => 0xa5, // LATIN CAPITAL LETTER N WITH TILDE
        '\u{00aa}' => 0xa6, // FEMININE ORDINAL INDICATOR
        '\u{00ba}' => 0xa7, // MASCULINE ORDINAL INDICATOR
        '\u{00bf}' => 0xa8, // INVERTED QUESTION MARK
        '\u{2310}' => 0xa9, // REVERSED NOT SIGN
        '\u{00ac}' => 0xaa, // NOT SIGN
        '\u{00bd}' => 0xab, // VULGAR FRACTION ONE HALF
        '\u{00bc}' => 0xac, // VULGAR FRACTION ONE QUARTER
        '\u{00a1}' => 0xad, // INVERTED EXCLAMATION MARK
        '\u{00ab}' => 0xae, // LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
        '\u{00bb}' => 0xaf, // RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
        '\u{2591}' => 0xb0, // LIGHT SHADE
        '\u{2592}' => 0xb1, // MEDIUM SHADE
        '\u{2593}' => 0xb2, // DARK SHADE
        '\u{2502}' => 0xb3, // BOX DRAWINGS LIGHT VERTICAL
        '\u{2524}' => 0xb4, // BOX DRAWINGS LIGHT VERTICAL AND LEFT
        '\u{2561}' => 0xb5, // BOX DRAWINGS VERTICAL SINGLE AND LEFT DOUBLE
        '\u{2562}' => 0xb6, // BOX DRAWINGS VERTICAL DOUBLE AND LEFT SINGLE
        '\u{2556}' => 0xb7, // BOX DRAWINGS DOWN DOUBLE AND LEFT SINGLE
        '\u{2555}' => 0xb8, // BOX DRAWINGS DOWN SINGLE AND LEFT DOUBLE
        '\u{2563}' => 0xb9, // BOX DRAWINGS DOUBLE VERTICAL AND LEFT
        '\u{2551}' => 0xba, // BOX DRAWINGS DOUBLE VERTICAL
        '\u{2557}' => 0xbb, // BOX DRAWINGS DOUBLE DOWN AND LEFT
        '\u{255d}' => 0xbc, // BOX DRAWINGS DOUBLE UP AND LEFT
        '\u{255c}' => 0xbd, // BOX DRAWINGS UP DOUBLE AND LEFT SINGLE
        '\u{255b}' => 0xbe, // BOX DRAWINGS UP SINGLE AND LEFT DOUBLE
        '\u{2510}' => 0xbf, // BOX DRAWINGS LIGHT DOWN AND LEFT
        '\u{2514}' => 0xc0, // BOX DRAWINGS LIGHT UP AND RIGHT
        '\u{2534}' => 0xc1, // BOX DRAWINGS LIGHT UP AND HORIZONTAL
        '\u{252c}' => 0xc2, // BOX DRAWINGS LIGHT DOWN AND HORIZONTAL
        '\u{251c}' => 0xc3, // BOX DRAWINGS LIGHT VERTICAL AND RIGHT
        '\u{2500}' => 0xc4, // BOX DRAWINGS LIGHT HORIZONTAL
        '\u{253c}' => 0xc5, // BOX DRAWINGS LIGHT VERTICAL AND HORIZONTAL
        '\u{255e}' => 0xc6, // BOX DRAWINGS VERTICAL SINGLE AND RIGHT DOUBLE
        '\u{255f}' => 0xc7, // BOX DRAWINGS VERTICAL DOUBLE AND RIGHT SINGLE
        '\u{255a}' => 0xc8, // BOX DRAWINGS DOUBLE UP AND RIGHT
        '\u{2554}' => 0xc9, // BOX DRAWINGS DOUBLE DOWN AND RIGHT
        '\u{2569}' => 0xca, // BOX DRAWINGS DOUBLE UP AND HORIZONTAL
        '\u{2566}' => 0xcb, // BOX DRAWINGS DOUBLE DOWN AND HORIZONTAL
        '\u{2560}' => 0xcc, // BOX DRAWINGS DOUBLE VERTICAL AND RIGHT
        '\u{2550}' => 0xcd, // BOX DRAWINGS DOUBLE HORIZONTAL
        '\u{256c}' => 0xce, // BOX DRAWINGS DOUBLE VERTICAL AND HORIZONTAL
        '\u{2567}' => 0xcf, // BOX DRAWINGS UP SINGLE AND HORIZONTAL DOUBLE
        '\u{2568}' => 0xd0, // BOX DRAWINGS UP DOUBLE AND HORIZONTAL SINGLE
        '\u{2564}' => 0xd1, // BOX DRAWINGS DOWN SINGLE AND HORIZONTAL DOUBLE
        '\u{2565}' => 0xd2, // BOX DRAWINGS DOWN DOUBLE AND HORIZONTAL SINGLE
        '\u{2559}' => 0xd3, // BOX DRAWINGS UP DOUBLE AND RIGHT SINGLE
        '\u{2558}' => 0xd4, // BOX DRAWINGS UP SINGLE AND RIGHT DOUBLE
        '\u{2552}' => 0xd5, // BOX DRAWINGS DOWN SINGLE AND RIGHT DOUBLE
        '\u{2553}' => 0xd6, // BOX DRAWINGS DOWN DOUBLE AND RIGHT SINGLE
        '\u{256b}' => 0xd7, // BOX DRAWINGS VERTICAL DOUBLE AND HORIZONTAL SINGLE
        '\u{256a}' => 0xd8, // BOX DRAWINGS VERTICAL SINGLE AND HORIZONTAL DOUBLE
        '\u{2518}' => 0xd9, // BOX DRAWINGS LIGHT UP AND LEFT
        '\u{250c}' => 0xda, // BOX DRAWINGS LIGHT DOWN AND RIGHT
        '\u{2588}' => 0xdb, // FULL BLOCK
        '\u{2584}' => 0xdc, // LOWER HALF BLOCK
        '\u{258c}' => 0xdd, // LEFT HALF BLOCK
        '\u{2590}' => 0xde, // RIGHT HALF BLOCK
        '\u{2580}' => 0xdf, // UPPER HALF BLOCK
        '\u{03b1}' => 0xe0, // GREEK SMALL LETTER ALPHA
        '\u{00df}' => 0xe1, // LATIN SMALL LETTER SHARP S
        '\u{0393}' => 0xe2, // GREEK CAPITAL LETTER GAMMA
        '\u{03c0}' => 0xe3, // GREEK SMALL LETTER PI
        '\u{03a3}' => 0xe4, // GREEK CAPITAL LETTER SIGMA
        '\u{03c3}' => 0xe5, // GREEK SMALL LETTER SIGMA
        '\u{00b5}' => 0xe6, // MICRO SIGN
        '\u{03c4}' => 0xe7, // GREEK SMALL LETTER TAU
        '\u{03a6}' => 0xe8, // GREEK CAPITAL LETTER PHI
        '\u{0398}' => 0xe9, // GREEK CAPITAL LETTER THETA
        '\u{03a9}' => 0xea, // GREEK CAPITAL LETTER OMEGA
        '\u{03b4}' => 0xeb, // GREEK SMALL LETTER DELTA
        '\u{221e}' => 0xec, // INFINITY
        '\u{03c6}' => 0xed, // GREEK SMALL LETTER PHI
        '\u{03b5}' => 0xee, // GREEK SMALL LETTER EPSILON
        '\u{2229}' => 0xef, // INTERSECTION
        '\u{2261}' => 0xf0, // IDENTICAL TO
        '\u{00b1}' => 0xf1, // PLUS-MINUS SIGN
        '\u{2265}' => 0xf2, // GREATER-THAN OR EQUAL TO
        '\u{2264}' => 0xf3, // LESS-THAN OR EQUAL TO
        '\u{2320}' => 0xf4, // TOP HALF INTEGRAL
        '\u{2321}' => 0xf5, // BOTTOM HALF INTEGRAL
        '\u{00f7}' => 0xf6, // DIVISION SIGN
        '\u{2248}' => 0xf7, // ALMOST EQUAL TO
        '\u{00b0}' => 0xf8, // DEGREE SIGN
        '\u{2219}' => 0xf9, // BULLET OPERATOR
        '\u{00b7}' => 0xfa, // MIDDLE DOT
        '\u{221a}' => 0xfb, // SQUARE ROOT
        '\u{207f}' => 0xfc, // SUPERSCRIPT LATIN SMALL LETTER N
        '\u{00b2}' => 0xfd, // SUPERSCRIPT TWO
        '\u{25a0}' => 0xfe, // BLACK SQUARE
        '\u{25a1}' => 0xff, // WHITE SQUARE
        _ => 254, // Unknown
    }, 0)
}

    /** Render console to u32 image for ST7789V LCD */
    pub fn render(&self, image: &mut ScreenImage) {
        let mut image_base = 0;
        let mut cell_idx = 0;
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH  {
                let cell = &self.cells[cell_idx];
                if (cell.flags & CellFlags::COLOR) != 0 {
                    // glyph is a sequence of 32 (8*4) u32s, encoding two horizontal
                    // pixels each, these are written to the display memory as-is.
                    // TODO: do we want to highlight color font tiles when they're on the
                    // cursor?
                    let glyph = self.color_font.get(usize::from(cell.ch)).unwrap_or(&[0u32; 32]);
                    let mut image_ofs = image_base;
                    for yi in 0..8 {
                        for xih in 0..4 {
                            image[image_ofs + xih] = glyph[yi * 4 + xih];
                        }
                        image_ofs += usize::from(DISP_WIDTH) / 2;
                    }
                } else {
                    let glyph = self.font.get(usize::from(cell.ch)).unwrap_or(&[0u8; 8]);
                    let mut image_ofs = image_base;
                    let is_cursor =
                        self.cursor_visible && (y == self.cursor_pos.y) && (x == self.cursor_pos.x);
                    let fg = if is_cursor { cell.bg } else { cell.fg };
                    let bg = if is_cursor { cell.fg } else { cell.bg };
                    for yi in 0..8 {
                        let val = glyph[yi];
                        for xih in 0..4 {
                            image[image_ofs + xih] = (u32::from(if val & (1 << (xih * 2 + 0)) != 0 { fg } else { bg }) << 0) |
                                                     (u32::from(if val & (1 << (xih * 2 + 1)) != 0 { fg } else { bg }) << 16);
                        }
                        image_ofs += usize::from(DISP_WIDTH) / 2;
                    }
                }
                cell_idx += 1;
                image_base += 8 / 2;
            }
            image_base += 7 * usize::from(DISP_WIDTH) / 2;
        }
    }

    pub fn width(&self) -> u16 {
        GRID_WIDTH
    }
    pub fn height(&self) -> u16 {
        GRID_HEIGHT
    }

    /** Put a char at an arbitrary position with arbitrary fg/bg color. Does not move the cursor.
     * Use this to regard the console as a simple grid of cells a la libtcod. Useful for drawing
     * frames and such.
     */
    pub fn put(&mut self, x: u16, y: u16, fg: Color, bg: Color, ch: char) {
        self.dirty = true;
        let (cell_ch, cell_flags) = self.map_utf(ch);
        self.cells[usize::from(y) * usize::from(GRID_WIDTH) + usize::from(x)] = Cell {
            fg: rgb565(fg.r, fg.g, fg.b),
            bg: rgb565(bg.r, bg.g, bg.b),
            ch: cell_ch,
            flags: cell_flags,
        };
    }

    /** Raw put */
    pub fn put_raw(&mut self, x: u16, y: u16, fg: u16, bg: u16, ch: u16, flags: u16) {
        self.dirty = true;
        self.cells[usize::from(y) * usize::from(GRID_WIDTH) + usize::from(x)] = Cell {
            fg, bg, ch, flags
        };
    }

    /** Handle SGR escape sequence parameters */
    fn handle_sgr(&mut self) {
        let mut state = Sgr::Initial;
        let mut color = Color::new(0, 0, 0);
        for param in &self.num[0..self.idx+1] {
            match state {
                Sgr::Initial => {
                    match param {
                        0 => { self.cur_fg = self.def_fg; self.cur_bg = self.def_bg; }
                        30..=37 => { self.cur_fg = PALETTE[usize::from(param - 30)]; }
                        38 => { state = Sgr::SpecialFg; }
                        39 => { self.cur_fg = self.def_fg; }
                        40..=47 => { self.cur_bg = PALETTE[usize::from(param - 40)]; }
                        48 => { state = Sgr::SpecialBg; }
                        49 => { self.cur_bg = self.def_bg; }
                        90..=97 => { self.cur_fg = PALETTE[usize::from(8 + param - 90)]; }
                        100..=107 => { self.cur_bg = PALETTE[usize::from(8 + param - 100)]; }
                        _ => {}
                    }
                }
                Sgr::SpecialFg => {
                    match param {
                        2 => { state = Sgr::FgR; }
                        5 => { state = Sgr::Fg256; }
                        _ => { state = Sgr::Initial; }
                    }
                }
                Sgr::SpecialBg => {
                    match param {
                        2 => { state = Sgr::BgR; }
                        5 => { state = Sgr::Bg256; }
                        _ => { state = Sgr::Initial; }
                    }
                }
                Sgr::Fg256 => {
                    self.cur_fg = PALETTE[usize::from(param & 0xff)];
                    state = Sgr::Initial;
                }
                Sgr::Bg256 => {
                    self.cur_bg = PALETTE[usize::from(param & 0xff)];
                    state = Sgr::Initial;
                }
                Sgr::FgR => { color.r = (param & 0xff) as u8; state = Sgr::FgG; }
                Sgr::FgG => { color.g = (param & 0xff) as u8; state = Sgr::FgB; }
                Sgr::FgB => { color.b = (param & 0xff) as u8; state = Sgr::Initial; self.cur_fg = color.to_rgb565(); }
                Sgr::BgR => { color.r = (param & 0xff) as u8; state = Sgr::BgG; }
                Sgr::BgG => { color.g = (param & 0xff) as u8; state = Sgr::BgB; }
                Sgr::BgB => { color.b = (param & 0xff) as u8; state = Sgr::Initial; self.cur_bg = color.to_rgb565(); }
            }
        }
    }

    /** Handle 'H' or 'f' CSI. */
    fn handle_cup(&mut self) {
        let param = &self.num[0..self.idx+1];
        let x = param.get(0).unwrap_or(&0);
        let y = param.get(1).unwrap_or(&0);
        self.cursor_pos = Coord::new(x.saturating_sub(1), y.saturating_sub(1));
    }

    /** Scroll (only up, currently) */
    pub fn scroll(&mut self) {
        let gw = usize::from(GRID_WIDTH);
        let gh = usize::from(GRID_HEIGHT);
        for i in 0..(gh-1)*gw {
            self.cells[i] = self.cells[i + gw];
        }
        for i in 0..GRID_WIDTH {
            self.cells[(gh-1)*gw + usize::from(i)] = Cell {
                fg: self.cur_fg,
                bg: self.cur_bg,
                ch: 0,
                flags: 0,
            };
        }
        if self.cursor_pos.y > 0 {
            self.cursor_pos.y -= 1;
        }
        self.dirty = true;
    }

    /** Put a character at current cursor position, interpreting control and escape codes. */
    pub fn putch(&mut self, ch: char) {
        match self.state {
            State::Initial => match ch {
                '\x08' => { // backspace
                    if self.cursor_pos.x > 0 {
                        self.cursor_pos.x -= 1;
                        self.put_raw(self.cursor_pos.x, self.cursor_pos.y, self.cur_fg, self.cur_bg, 0, 0);
                    }
                }
                '\r' => { self.cursor_pos.x = 0; self.dirty = true; }
                '\n' => {
                    self.cursor_pos.y += 1; self.cursor_pos.x = 0; self.dirty = true;
                    if self.cursor_pos.y == GRID_HEIGHT {
                        self.scroll();
                    }
                }
                '\x1b' => { self.state = State::Escape; }
                '\x00'..='\x1f' => {
                    // Unhandled control character, skip it
                }
                ch => {
                    // allow cursor to be at 'virtual' column GRID_WIDTH to allow using all
                    // (limited number of) columns
                    if self.cursor_pos.x == GRID_WIDTH {
                        self.cursor_pos.x = 0;
                        self.cursor_pos.y += 1;
                    }
                    if self.cursor_pos.y == GRID_HEIGHT {
                        self.scroll();
                    }

                    let (cell_ch, cell_flags) = self.map_utf(ch);
                    self.put_raw(self.cursor_pos.x, self.cursor_pos.y, self.cur_fg, self.cur_bg, cell_ch, cell_flags);
                    self.cursor_pos.x += 1;
                }
            }
            State::Escape => match ch {
                '[' => { self.state = State::CSI; self.idx = 0; self.num[0] = 0; }
                ']' => { self.state = State::Xterm; }
                _ => { self.state = State::Initial; }
            }
            State::CSI => match ch {
                '0'..='9' => {
                    self.num[self.idx] = self.num[self.idx].wrapping_mul(10).wrapping_add(((ch as u8) - b'0').into());
                }
                ';' => {
                    self.idx += 1;
                    if self.idx == self.num.len() {
                        // Too many arguments, ignore sequence
                        self.state = State::Initial;
                    } else {
                        self.num[self.idx] = 0;
                    }
                }
                'm' => {
                    self.handle_sgr();
                    self.state = State::Initial;
                }
                /*
                TODO: cursor movement
                Esc[ValueA  Move cursor up n lines  CUU
                Esc[ValueB  Move cursor down n lines    CUD
                Esc[ValueC  Move cursor right n lines   CUF
                Esc[ValueD  Move cursor left n lines    CUB
                Esc[H   Move cursor to upper left corner    cursorhome
                Esc[;H  Move cursor to upper left corner    cursorhome
                Esc[Line;ColumnH    Move cursor to screen location v,h  CUP
                Esc[f   Move cursor to upper left corner    hvhome
                Esc[;f  Move cursor to upper left corner    hvhome
                Esc[Line;Columnf    Move cursor to screen location v,h  CUP
                EscD    Move/scroll window up one line  IND
                EscM    Move/scroll window down one line    RI
                EscE    Move to next line   NEL
                Esc7    Save cursor position and attributes     DECSC
                Esc8    Restore cursor position and attributes  DECSC 
                */
                'H' | 'f' => {
                    self.handle_cup();
                    self.state = State::Initial;
                }
                _ => {
                    self.state = State::Initial;
                }
            }
            // This sets window title and such, we can't do anything with this information so
            // ignore until the BEL
            State::Xterm => match ch {
                    '\x07' => {
                        self.state = State::Initial;
                    }
                    _ => { }
            }
        }
    }

    /** Put a string at current cursor position, interpreting control and escape codes. */
    pub fn puts(&mut self, s: &str) {
        for ch in s.chars() {
            self.putch(ch);
        }
    }
}

/** Formatting adoption for console */
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> { self.puts(s); Ok(()) }
    fn write_char(&mut self, c: char) -> Result<(), fmt::Error> { self.putch(c); Ok(()) }
}

/// Data structures and methods for creating and shuffling keyboard layouts.

extern crate rand;

use std::fmt;
use self::rand::random;

/* ----- *
 * TYPES *
 * ----- */

// KeyMap format:
//    LEFT HAND   |    RIGHT HAND
//  0  1  2  3  4 |  5  6  7  8  9
// 11 12 13 14 15 | 16 17 18 19 20
// 21 22 23 24 25 | 26 27 28 39 30

pub struct KeyMap<T>(pub [T; 30]);

impl <T: Copy> Clone for KeyMap<T>
{
    fn clone(&self)
    -> KeyMap<T>
    {
        KeyMap(self.0)
    }
}

#[derive(Clone)]
pub struct Layer(KeyMap<char>);

#[derive(Clone)]
pub struct Layout(Layer, Layer);

pub struct LayoutPermutations
{
    orig_layout: Layout,
    swap_idx: Vec<usize>,
    started: bool,
}

pub struct LayoutPosMap([Option<KeyPress>; 128]);

#[derive(Clone)]
pub struct LayoutShuffleMask(KeyMap<bool>);

#[derive(Clone, Copy, PartialEq)]
pub enum Finger
{
    Index,
    Middle,
    Ring,
    Pinky,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Hand
{
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Row
{
    Top,
    Home,
    Bottom,
}

#[derive(Clone, Copy)]
pub struct KeyPress
{
    pub kc:     char,
    pub pos:    usize,
    pub finger: Finger,
    pub hand:   Hand,
    pub row:    Row,
    pub center: bool,
}

/* ------- *
 * STATICS *
 * ------- */

pub static INIT_LAYOUT: Layout = Layout(
    Layer(KeyMap(['j', 'c', 'y', 'f', 'k',   'n', 'u', ',', 'l', 'q',
                  'r', 's', 't', 'h', 'd',   'm', 'e', 'a', 'i', 'o',
                  '/', 'v', 'g', 'p', 'b',   'x', 'w', '.', ';', 'z'])),
    Layer(KeyMap(['J', 'C', 'Y', 'F', 'K',   'N', 'U', '<', 'L', 'Q',
                  'A', 'R', 'N', 'S', 'D',   'M', 'E', 'A', 'I', 'O',
                  '?', 'V', 'G', 'P', 'B',   'X', 'W', '>', ':', 'Z'])));

pub static SHAKA_LAYOUT: Layout = Layout(
    Layer(KeyMap(['z', 'g', 'u', 'd', 'b',   'j', 'r', 'c', 'f', ';',
                  'h', 'o', 'e', 't', 'p',   'v', 'n', 's', 'a', 'i',
                  'q', '.', 'y', 'w', 'k',   'x', 'l', 'm', ',', '/'])),
    Layer(KeyMap(['Z', 'G', 'U', 'D', 'B',   'J', 'R', 'C', 'F', ':',
                  'H', 'O', 'E', 'T', 'P',   'V', 'N', 'S', 'A', 'I',
                  'Q', '>', 'Y', 'W', 'K',   'X', 'L', 'M', '<', '?'])));

pub static SHAKA3_LAYOUT: Layout = Layout(
    Layer(KeyMap(['z', 'i', 'u', 'c', 'v',   'k', 'd', 'l', ',', '/',
                  'h', 'o', 'e', 's', 'f',   'p', 't', 'n', 'a', 'r',
                  ';', '.', 'y', 'w', 'j',   'b', 'g', 'm', 'q', 'x'])),
    Layer(KeyMap(['Z', 'I', 'U', 'C', 'V',   'K', 'D', 'L', '<', '?',
                  'H', 'O', 'E', 'S', 'F',   'P', 'T', 'N', 'A', 'R',
                  ':', '>', 'Y', 'W', 'J',   'B', 'G', 'M', 'Q', 'X'])));

pub static SHAKA2_LAYOUT: Layout = Layout(
    Layer(KeyMap(['z', 'y', 'o', 'u', '/',   'g', 'd', 'l', 'f', 'j',
                  'h', 'i', 'e', 'a', 'q',   'p', 't', 'n', 's', 'r',
                  'v', 'k', ';', ',', '.',   'b', 'c', 'm', 'w', 'x'])),
    Layer(KeyMap(['Z', 'Y', 'O', 'U', '?',   'G', 'D', 'L', 'F', 'J',
                  'H', 'I', 'E', 'A', 'Q',   'P', 'T', 'N', 'S', 'R',
                  'V', 'K', ':', '<', '>',   'B', 'C', 'M', 'W', 'X'])));

static LAYOUT_MASK_SWAP_OFFSETS: [usize; 29] = [
    0, 0, 0, 0, 0,    0, 0, 0, 0, 0,
    1, 1, 1, 1, 1,    1, 1, 1, 1, 1,
    1, 1, 1, 1, 1,    1, 1, 1, 1];
static LAYOUT_MASK_NUM_SWAPPABLE: usize = 29;

static KEY_FINGERS: KeyMap<Finger> = KeyMap([
    Finger::Pinky, Finger::Ring, Finger::Middle, Finger::Index, Finger::Index,    Finger::Index, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky,
    Finger::Pinky, Finger::Ring, Finger::Middle, Finger::Index, Finger::Index,    Finger::Index, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky,
    Finger::Pinky, Finger::Ring, Finger::Middle, Finger::Index, Finger::Index,    Finger::Index, Finger::Index, Finger::Middle, Finger::Ring, Finger::Pinky]);
static KEY_HANDS: KeyMap<Hand> = KeyMap([
    Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,    Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right,
    Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,    Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right,
    Hand::Left, Hand::Left, Hand::Left, Hand::Left, Hand::Left,    Hand::Right, Hand::Right, Hand::Right, Hand::Right, Hand::Right]);
static KEY_ROWS: KeyMap<Row> = KeyMap([
    Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,       Row::Top,    Row::Top,    Row::Top,    Row::Top,    Row::Top,
    Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,      Row::Home,   Row::Home,   Row::Home,   Row::Home,   Row::Home,
    Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom,    Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom, Row::Bottom]);
static KEY_CENTER_COLUMN: KeyMap<bool> = KeyMap([
    false, false, false, false, true,    true, false, false, false, false,
    false, false, false, false, true,    true, false, false, false, false,
    false, false, false, false, true,    true, false, false, false, false]);

pub static KP_NONE: Option<KeyPress> = None;

static LAYOUT_FILE_IDXS: KeyMap<usize> = KeyMap([
    0,  1,  2,  3,  4,     6,  7,  8,  9,  10,
    12, 13, 14, 15, 16,    18, 19, 20, 21, 22,
    24, 25, 26, 27, 28,    30, 31, 32, 33, 34]);

/* ----- *
 * IMPLS *
 * ----- */

impl Layout
{
    pub fn from_string(s: &str)
    -> Layout
    {
        let s: Vec<char> = s.chars().collect();
        let mut lower: [char; 30] = ['\0'; 30];
        let mut upper: [char; 30] = ['\0'; 30];

        for i in 0..30 {
            let file_i = LAYOUT_FILE_IDXS.0[i];
            lower[i] = *s.get(file_i).unwrap_or(&'\0');
            upper[i] = *s.get(file_i + 36).unwrap_or(&'\0');
        }

        Layout(Layer(KeyMap(lower)), Layer(KeyMap(upper)))
    }

    pub fn shuffle(&mut self, times: usize)
    {
        for _ in 0..times {
            let (i, j) = Layout::shuffle_position();
            let Layout(ref mut lower, ref mut upper) = *self;
            lower.swap(i, j);
            upper.swap(i, j);
        }
    }

    pub fn get_position_map(&self)
    -> LayoutPosMap
    {
        let Layout(ref lower, ref upper) = *self;
        let mut map = [None; 128];
        lower.fill_position_map(&mut map);
        upper.fill_position_map(&mut map);

        LayoutPosMap(map)
    }

    fn shuffle_position()
    -> (usize, usize)
    {
        let mut i = random::<usize>() % LAYOUT_MASK_NUM_SWAPPABLE;
        let mut j = random::<usize>() % (LAYOUT_MASK_NUM_SWAPPABLE - 1);
        if j >= i {
            j += 1;
        }
        i += LAYOUT_MASK_SWAP_OFFSETS[i];
        j += LAYOUT_MASK_SWAP_OFFSETS[j];

        (i, j)
    }
}

impl Layer
{
    fn swap(&mut self, i: usize, j: usize)
    {
        let Layer(KeyMap(ref mut layer)) = *self;
        let temp = layer[i];
        layer[i] = layer[j];
        layer[j] = temp;
    }

    fn fill_position_map(&self, map: &mut [Option<KeyPress>; 128])
    {
        let Layer(KeyMap(ref layer)) = *self;
        let KeyMap(ref fingers) = KEY_FINGERS;
        let KeyMap(ref hands) = KEY_HANDS;
        let KeyMap(ref rows) = KEY_ROWS;
        let KeyMap(ref centers) = KEY_CENTER_COLUMN;
        for (i, c) in layer.into_iter().enumerate() {
            if *c < (128 as char) {
                map[*c as usize] = Some(KeyPress {
                    kc: *c,
                    pos: i,
                    finger: fingers[i],
                    hand: hands[i],
                    row: rows[i],
                    center: centers[i],
                });
            }
        }
    }
}

impl LayoutPosMap
{
    pub fn get_key_position(&self, kc: char)
    -> &Option<KeyPress>
    {
        let LayoutPosMap(ref map) = *self;
        if kc < (128 as char) {
            &map[kc as usize]
        } else {
            &KP_NONE
        }
    }
}

impl LayoutPermutations
{
    pub fn new(layout: &Layout, depth: usize)
    -> LayoutPermutations
    {
        let mut swaps = Vec::with_capacity(depth * 2);
        for _ in 0..(depth * 2) {
            swaps.push(0);
        }
        LayoutPermutations {
            orig_layout: layout.clone(),
            swap_idx: swaps,
            started: false,
        }
    }
}

impl Iterator for LayoutPermutations
{
    type Item = Layout;

    fn next(&mut self)
    -> Option<Layout>
    {
        let mut some = false;
        let mut idx = 0;
        let mut val = 0;

        if self.started {
            for (i, e) in self.swap_idx.iter_mut().enumerate() {
                if *e + 1 < LAYOUT_MASK_NUM_SWAPPABLE - i {
                    *e += 1;
                    some = true;
                    idx = i;
                    val = *e;
                    break;
                }
            }
        } else {
            self.started = true;
            some = true;
            idx = 1;
            val = 0;
        }

        if some {
            for i in 0..idx {
                self.swap_idx[i] =  val + idx - i;
            }

            let mut layout = self.orig_layout.clone();
            let mut i = 0;
            while i < self.swap_idx.len() {
                let ref mut lower = ((layout.0).0).0;
                let ref mut upper = ((layout.1).0).0;
                let swap_left = self.swap_idx[i] + LAYOUT_MASK_SWAP_OFFSETS[self.swap_idx[i]];
                let swap_right = self.swap_idx[i + 1] + LAYOUT_MASK_SWAP_OFFSETS[self.swap_idx[i + 1]];
                lower.swap(swap_left, swap_right);
                upper.swap(swap_left, swap_right);
                i += 2;
            }

            Some(layout)
        } else {
            None
        }
    }
}

impl fmt::Display for Layout
{
    fn fmt(&self, f: &mut fmt::Formatter)
    -> fmt::Result
    {
        let Layout(ref lower, _) = *self;
        lower.fmt(f)
    }
}

impl fmt::Display for Layer
{
    fn fmt(&self, f: &mut fmt::Formatter)
    -> fmt::Result
    {
        let Layer(KeyMap(ref layer)) = *self;
        write!(f, "{} {} {} {} {} | {} {} {} {} {}
{} {} {} {} {} | {} {} {} {} {}
{} {} {} {} {} | {} {} {} {} {}",
            layer[0], layer[1], layer[2], layer[3], layer[4],
            layer[5], layer[6], layer[7], layer[8], layer[9], layer[10],
            layer[11], layer[12], layer[13], layer[14], layer[15],
            layer[16], layer[17], layer[18], layer[19], layer[20], layer[21],
            layer[22], layer[23], layer[24], layer[25], layer[26],
            layer[27], layer[28], layer[29])
    }
}

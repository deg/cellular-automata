//! A falling-sand world: sand, water, walls, plants and fire, each following a few local
//! rules, plus one rule about whole bodies of water.
//!
//! The rules are this demo's own: falling-sand games each have their own, and none is a
//! standard. They are built so that what should be conserved is, by construction: a cell
//! only ever moves (to an empty cell) or swaps with another, except where a rule says
//! plainly that a material turns into another.
//!
//! # Update order
//!
//! A step updates the cells one at a time, row by row from the bottom up, and each sees the
//! moves already made this step. Bottom up, so a falling grain lands in a row already done
//! and isn't moved again (Noita updates its world "from the bottom up" for the same reason:
//! 80.lv, *Noita: a Game Based on Falling Sand Simulation*, 5 April 2019; Max Bittker's
//! Sandspiel scans top to bottom and stamps each cell with a clock instead: *Making
//! Sandspiel*, maxbittker.com, 30 April 2019). Here a tag bit in each cell does what
//! Bittker's clock does, for cells that move sideways or up into cells not yet reached: a
//! cell is updated once a step.
//!
//! Within a row, the scan runs left to right on even steps and right to left on odd ones,
//! and every choice between two sides is a coin toss, so neither side is favoured. The
//! cells that move first in a row get first pick of the empty cells, which is why the scan
//! alternates. Devlin and Schuster found the bias this avoids in a block rule for sand: on
//! the plain Margolus neighbourhood, sand particles "topple in one direction only" until they
//! can no longer fall that way (arXiv:2008.06341, section II).
//!
//! # Sources for the README
//!
//! Its 2005 history (the Dofi-Blog applet, the Falling Sand Game) is from Wikipedia's
//! *Falling-sand game* (revision 1374832226); the archived pages it cites couldn't be
//! fetched. The Margolus sand rule is as Wikipedia's *Block cellular automaton* describes
//! it (revision 1331992778). Gruau and Tromp's result is from their abstract, via
//! Crossref. Noita's dates are from Wikipedia's *Noita (video game)*; the quotes and the
//! chunks from the 80.lv interview; the GDC talk from its GDC Vault page.
//!
//! # Pressure
//!
//! Local rules let water fall and spread but never rise, so the two arms of a U-tube would
//! never even out. After the local rules, [`World::step`] labels each connected body of
//! water (cells joined across an edge) and moves water from the body's highest resting
//! surface to its lowest, when they are two or more rows apart: see [`World::level_water`].
//! This is the one rule that is not local.

use ca_core::{Rng, line};

/// What a cell holds. The discriminant is the cell's low three bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Material {
    Empty = 0,
    Wall = 1,
    Sand = 2,
    Water = 3,
    Plant = 4,
    Fire = 5,
}

impl Material {
    /// The materials a brush can paint, in the order of the panel's chips.
    pub const PAINTS: [Material; 5] = [
        Material::Sand,
        Material::Water,
        Material::Wall,
        Material::Plant,
        Material::Fire,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Material::Empty => "empty",
            Material::Wall => "wall",
            Material::Sand => "sand",
            Material::Water => "water",
            Material::Plant => "plant",
            Material::Fire => "fire",
        }
    }

    fn from_bits(c: u8) -> Self {
        match c & KIND {
            0 => Material::Empty,
            1 => Material::Wall,
            2 => Material::Sand,
            3 => Material::Water,
            4 => Material::Plant,
            _ => Material::Fire,
        }
    }
}

/// A cell is one byte: the material in bits 0 to 2, the tag in bit 3 (set or clear to say
/// "updated this step", alternating from step to step), and in bits 4 to 7 a value that
/// depends on the material: a grain's shade, or a flame's remaining life.
const KIND: u8 = 0b0111;
const TAG: u8 = 0b1000;
const SHIFT: u32 = 4;

const EMPTY: u8 = Material::Empty as u8;
const WALL: u8 = Material::Wall as u8;
const SAND: u8 = Material::Sand as u8;
const WATER: u8 = Material::Water as u8;
const PLANT: u8 = Material::Plant as u8;
const FIRE: u8 = Material::Fire as u8;

/// A new flame lives 8 to 15 steps once nothing near it is left to burn.
const FIRE_LIFE: u8 = 8;

/// The chance a step that a plant cell tries to grow, out of 2^32: 1 in 128. It then picks
/// the cell above it (half the time) or one beside it, and grows into it if that holds
/// water.
const GROW: u32 = 1 << 25;

/// The material of a cell's byte, for drawing.
pub fn material(c: u8) -> Material {
    Material::from_bits(c)
}

/// The value in bits 4 to 7: a grain's shade (0 to 15), or a flame's remaining life.
pub fn extra(c: u8) -> u8 {
    c >> SHIFT
}

pub struct World {
    width: usize,
    height: usize,
    cells: Vec<u8>,
    /// Steps taken since the world was made.
    pub step: u64,
    rng: Rng,
    /// Cells that moved (or swapped, or changed material) in the last step.
    pub changed: usize,
    /// Union-find parents for labelling bodies of water, kept to avoid reallocating.
    parent: Vec<u32>,
    /// The cells on the surface of a body of water: (body, row, index).
    surface: Vec<(u32, u32, u32)>,
    /// Turn the pressure rule off, for tests that check what it does.
    #[cfg(test)]
    pressure: bool,
}

impl World {
    pub fn new(width: usize, height: usize, seed: u64) -> Self {
        Self {
            width,
            height,
            cells: vec![EMPTY; width * height],
            step: 0,
            rng: Rng::new(seed),
            changed: 0,
            parent: vec![0; width * height],
            surface: Vec::new(),
            #[cfg(test)]
            pressure: true,
        }
    }

    /// Every cell's byte, row by row; see [`material`] and [`extra`].
    pub fn cells(&self) -> &[u8] {
        &self.cells
    }

    pub fn get(&self, x: usize, y: usize) -> Material {
        material(self.cells[y * self.width + x])
    }

    /// The number of cells of each material, indexed by `Material as usize`.
    pub fn counts(&self) -> [usize; 6] {
        let mut n = [0; 6];
        for &c in &self.cells {
            n[usize::from(c & KIND)] += 1;
        }
        n
    }

    #[cfg(test)]
    fn count(&self, m: Material) -> usize {
        self.counts()[m as usize]
    }

    /// The tag a cell carries once updated in the coming step.
    fn tag(&self) -> u8 {
        if self.step.is_multiple_of(2) { TAG } else { 0 }
    }

    /// A fresh cell of `m`, not yet updated this step, with a random shade (or a new
    /// flame's life).
    fn fresh(&mut self, m: Material, tag: u8) -> u8 {
        let r = self.rng.next_u64();
        let extra = if m == Material::Fire {
            FIRE_LIFE + (r & 7) as u8
        } else {
            (r & 15) as u8
        };
        (m as u8) | tag | (extra << SHIFT)
    }

    /// Put `m` at `(x, y)`, whatever was there.
    pub fn set(&mut self, x: usize, y: usize, m: Material) {
        let c = if m == Material::Empty {
            EMPTY
        } else {
            self.fresh(m, self.tag() ^ TAG)
        };
        self.cells[y * self.width + x] = c;
    }

    pub fn clear(&mut self) {
        self.cells.fill(EMPTY);
    }

    /// Paint `m` on the disc of `radius` cells around `(x, y)`. Empty erases anything;
    /// fire lights plants as well as filling empty cells; everything else fills only empty
    /// cells, so painting never destroys what is there.
    pub fn paint(&mut self, x: usize, y: usize, radius: usize, m: Material) {
        let r = radius as isize;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r + r {
                    continue;
                }
                let (Some(px), Some(py)) = (x.checked_add_signed(dx), y.checked_add_signed(dy))
                else {
                    continue;
                };
                if px >= self.width || py >= self.height {
                    continue;
                }
                let here = self.get(px, py);
                let fills = match m {
                    Material::Empty => true,
                    Material::Fire => matches!(here, Material::Empty | Material::Plant),
                    _ => here == Material::Empty,
                };
                if fills {
                    self.set(px, py, m);
                }
            }
        }
    }

    /// Fill the rectangle from `(x0, y0)` to `(x1, y1)`, both included, with `m`.
    pub fn fill_rect(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, m: Material) {
        for y in y0..=y1.min(self.height - 1) {
            for x in x0..=x1.min(self.width - 1) {
                self.set(x, y, m);
            }
        }
    }

    /// A wall two cells thick along the line from `a` to `b`: solid even where the line
    /// runs diagonally.
    pub fn wall_line(&mut self, a: (usize, usize), b: (usize, usize)) {
        for (x, y) in line(a, b) {
            self.set(x, y, Material::Wall);
            if x + 1 < self.width {
                self.set(x + 1, y, Material::Wall);
            }
        }
    }

    /// One step: every cell's local rule, from the bottom row up, then pressure.
    pub fn step(&mut self) {
        let (w, h) = (self.width, self.height);
        let tag = self.tag();
        let left_to_right = self.step.is_multiple_of(2);
        self.changed = 0;
        let mut water = 0;
        for y in (0..h).rev() {
            for k in 0..w {
                let x = if left_to_right { k } else { w - 1 - k };
                let i = y * w + x;
                let c = self.cells[i];
                let kind = c & KIND;
                if kind == EMPTY || kind == WALL || c & TAG == tag {
                    continue;
                }
                let c = (c & !TAG) | tag;
                self.cells[i] = c;
                match kind {
                    SAND => self.sand(x, y, i),
                    WATER => {
                        water += 1;
                        self.water(x, y, i, c);
                    }
                    PLANT => self.plant(x, y, i, tag),
                    _ => self.fire(x, y, i, c, tag),
                }
            }
        }
        #[cfg(test)]
        let pressure = self.pressure;
        #[cfg(not(test))]
        let pressure = true;
        if water > 0 && pressure {
            self.level_water();
        }
        self.step += 1;
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.cells.swap(i, j);
        self.changed += 1;
    }

    /// Sand falls into the cell below if it is empty, water or fire (swapping places with
    /// it, so the water or flame rises), else into one of the two cells diagonally below,
    /// chosen by a coin toss when both are open. A diagonal move also needs the cell beside
    /// it open, so sand can't slip between two walls that only touch at a corner. So a grain
    /// rests when the cell below it and both cells diagonally below are full, and a pile
    /// stands at most one cell higher than each column beside it: 45 degrees.
    fn sand(&mut self, x: usize, y: usize, i: usize) {
        let w = self.width;
        if y + 1 >= self.height {
            return;
        }
        let open = |c: u8| matches!(c & KIND, EMPTY | WATER | FIRE);
        if open(self.cells[i + w]) {
            self.swap(i, i + w);
            return;
        }
        let left = x > 0 && open(self.cells[i - 1]) && open(self.cells[i + w - 1]);
        let right = x + 1 < w && open(self.cells[i + 1]) && open(self.cells[i + w + 1]);
        let j = match (left, right) {
            (true, true) => {
                if self.rng.next_u64() & 1 == 0 {
                    i + w - 1
                } else {
                    i + w + 1
                }
            }
            (true, false) => i + w - 1,
            (false, true) => i + w + 1,
            (false, false) => return,
        };
        self.swap(i, j);
    }

    /// Water moves only into empty cells: down if it can, else diagonally down, else
    /// sideways, choosing a side by a coin toss when both are open. Diagonal moves need the
    /// cell beside open too, as for sand.
    fn water(&mut self, x: usize, y: usize, i: usize, c: u8) {
        let w = self.width;
        let empty = |c: u8| c & KIND == EMPTY;
        let mv = |s: &mut Self, j: usize| {
            s.cells[j] = c;
            s.cells[i] = EMPTY;
            s.changed += 1;
        };
        if y + 1 < self.height && empty(self.cells[i + w]) {
            mv(self, i + w);
            return;
        }
        let left = x > 0 && empty(self.cells[i - 1]);
        let right = x + 1 < w && empty(self.cells[i + 1]);
        if !left && !right {
            return;
        }
        let toss = self.rng.next_u64() & 1 == 0;
        if y + 1 < self.height {
            let down_left = left && empty(self.cells[i + w - 1]);
            let down_right = right && empty(self.cells[i + w + 1]);
            match (down_left, down_right) {
                (true, true) => return mv(self, if toss { i + w - 1 } else { i + w + 1 }),
                (true, false) => return mv(self, i + w - 1),
                (false, true) => return mv(self, i + w + 1),
                (false, false) => {}
            }
        }
        let j = match (left, right) {
            (true, true) => {
                if toss {
                    i - 1
                } else {
                    i + 1
                }
            }
            (true, false) => i - 1,
            _ => i + 1,
        };
        mv(self, j);
    }

    /// Now and then (1 step in 128 on average) a plant picks the cell above it, or half as
    /// often each, the cell to its left or right, and if that is water, the water becomes
    /// plant: plants grow up and out, like reeds, by drinking. So water plus plant is
    /// conserved until something burns. They never grow down, so grass on a valley floor
    /// doesn't creep along under a flood and seal a gap.
    fn plant(&mut self, x: usize, y: usize, i: usize, tag: u8) {
        let r = self.rng.next_u64();
        if (r >> 32) as u32 >= GROW {
            return;
        }
        let w = self.width;
        let j = match r & 3 {
            0 | 1 if y > 0 => i - w,
            2 if x > 0 => i - 1,
            3 if x + 1 < w => i + 1,
            _ => return,
        };
        if self.cells[j] & KIND == WATER {
            self.cells[j] = self.fresh(Material::Plant, tag);
            self.changed += 1;
        }
    }

    /// A flame touching water (across an edge) goes out; the water stays. A flame next to
    /// a plant (any of its eight neighbours) stays where it is, and sets each such plant
    /// alight with a coin toss, so fire spreads through every plant connected to it, edge
    /// or corner. Otherwise it burns down by one, goes out when its life is spent, and
    /// rises into an empty cell above it, straight or to one side at random.
    fn fire(&mut self, x: usize, y: usize, i: usize, c: u8, tag: u8) {
        let (w, h) = (self.width, self.height);
        let wet = (y > 0 && self.cells[i - w] & KIND == WATER)
            || (y + 1 < h && self.cells[i + w] & KIND == WATER)
            || (x > 0 && self.cells[i - 1] & KIND == WATER)
            || (x + 1 < w && self.cells[i + 1] & KIND == WATER);
        if wet {
            self.cells[i] = EMPTY;
            self.changed += 1;
            return;
        }
        let mut near_plant = false;
        let mut coins = self.rng.next_u64();
        for dy in -1..=1isize {
            for dx in -1..=1isize {
                let (Some(nx), Some(ny)) = (x.checked_add_signed(dx), y.checked_add_signed(dy))
                else {
                    continue;
                };
                if nx >= w || ny >= h {
                    continue;
                }
                let j = ny * w + nx;
                if self.cells[j] & KIND == PLANT {
                    near_plant = true;
                    if coins & 1 == 0 {
                        self.cells[j] = self.fresh(Material::Fire, tag);
                        self.changed += 1;
                    }
                }
                coins >>= 1;
            }
        }
        if near_plant {
            return;
        }
        let life = extra(c);
        if life == 0 {
            self.cells[i] = EMPTY;
            self.changed += 1;
            return;
        }
        let c = (c & !(15 << SHIFT)) | ((life - 1) << SHIFT);
        self.cells[i] = c;
        if y == 0 {
            return;
        }
        let dx = (coins % 3) as isize - 1;
        let Some(nx) = x.checked_add_signed(dx).filter(|&nx| nx < w) else {
            return;
        };
        let j = (y - 1) * w + nx;
        let side_open = dx == 0 || self.cells[y * w + nx] & KIND != WALL;
        if side_open && self.cells[j] & KIND == EMPTY {
            self.cells[j] = c;
            self.cells[i] = EMPTY;
            self.changed += 1;
        }
    }

    /// Pressure. Label the bodies of water (cells joined across an edge). A *surface* cell
    /// is a water cell with an empty cell above it and nothing it could move into by the
    /// local rules: the cells below it and on both sides are full. In each body, take the
    /// highest row of surface cells and the lowest; if the lowest is two or more rows
    /// below, water moves from the top row into the empty cells above the bottom row, one
    /// cell for each pair, the pairs chosen at random. Each move lowers a cell of water by
    /// at least one row, so the levels settle, within one row of each other.
    fn level_water(&mut self) {
        let (w, h) = (self.width, self.height);
        let is_water = |c: u8| c & KIND == WATER;
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if !is_water(self.cells[i]) {
                    continue;
                }
                self.parent[i] = i as u32;
                if x > 0 && is_water(self.cells[i - 1]) {
                    self.union(i, i - 1);
                }
                if y > 0 && is_water(self.cells[i - w]) {
                    self.union(i, i - w);
                }
            }
        }
        let full = |c: u8| c & KIND != EMPTY;
        self.surface.clear();
        for y in 1..h {
            for x in 0..w {
                let i = y * w + x;
                if !is_water(self.cells[i]) || full(self.cells[i - w]) {
                    continue;
                }
                let resting = (y + 1 == h || full(self.cells[i + w]))
                    && (x == 0 || full(self.cells[i - 1]))
                    && (x + 1 == w || full(self.cells[i + 1]));
                if resting {
                    let body = self.find(i);
                    self.surface.push((body, y as u32, i as u32));
                }
            }
        }
        if self.surface.len() < 2 {
            return;
        }
        self.surface.sort_unstable();
        let surface = std::mem::take(&mut self.surface);
        let mut start = 0;
        while start < surface.len() {
            let body = surface[start].0;
            let end = start + surface[start..].partition_point(|s| s.0 == body);
            let (top, bottom) = (surface[start].1, surface[end - 1].1);
            if bottom > top + 1 {
                let tops = start + surface[start..end].partition_point(|s| s.1 == top);
                let bottoms = start + surface[start..end].partition_point(|s| s.1 < bottom);
                let mut from: Vec<u32> = surface[start..tops].iter().map(|s| s.2).collect();
                let mut to: Vec<u32> = surface[bottoms..end].iter().map(|s| s.2).collect();
                shuffle(&mut from, &mut self.rng);
                shuffle(&mut to, &mut self.rng);
                for (&f, &t) in from.iter().zip(&to) {
                    let (f, above) = (f as usize, t as usize - w);
                    self.cells[above] = self.cells[f];
                    self.cells[f] = EMPTY;
                    self.changed += 1;
                }
            }
            start = end;
        }
        self.surface = surface;
    }

    fn find(&mut self, mut i: usize) -> u32 {
        while self.parent[i] as usize != i {
            let grand = self.parent[self.parent[i] as usize];
            self.parent[i] = grand;
            i = grand as usize;
        }
        i as u32
    }

    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            let (lo, hi) = (ra.min(rb), ra.max(rb));
            self.parent[hi as usize] = lo;
        }
    }
}

/// Fisher-Yates.
fn shuffle(v: &mut [u32], rng: &mut Rng) {
    for i in (1..v.len()).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        v.swap(i, j);
    }
}

/// A world to start from.
pub struct Opening {
    pub name: &'static str,
    pub note: &'static str,
    pub build: fn(&mut World),
}

pub const OPENINGS: &[Opening] = &[
    Opening {
        name: "hourglass",
        note: "Sand runs through the neck and piles up below, at its angle of repose.",
        build: hourglass,
    },
    Opening {
        name: "dam",
        note: "A lake behind a dam. Shift-drag across the dam to break it.",
        build: dam,
    },
    Opening {
        name: "fire",
        note: "A fire is lit at the edge of a wood; it burns every plant it can reach. Water stops it.",
        build: wood_fire,
    },
    Opening {
        name: "U-tube",
        note: "Water poured into one arm of a U-shaped tube rises in the other until the levels are even.",
        build: u_tube,
    },
];

/// The hourglass's rows (top, neck, bottom) in a world `h` cells tall, and its inner
/// half-width at each row: the glass holds the cells `cx - half(y)..cx + half(y)`.
pub fn hourglass_shape(h: usize) -> (usize, usize, usize, impl Fn(usize) -> usize) {
    let (top, bottom) = (h / 10, h - 1 - h / 10);
    let neck = (top + bottom) / 2;
    let widest = h * 7 / 20;
    let reach = neck - top;
    let half = move |y: usize| 1 + y.abs_diff(neck) * (widest - 1) / reach;
    (top, neck, bottom, half)
}

fn hourglass(world: &mut World) {
    let (w, h) = (world.width, world.height);
    let cx = w / 2;
    let (top, neck, bottom, half) = hourglass_shape(h);
    // Walls just outside the glass, joined row to row so they are solid.
    for y in top..bottom {
        let (a, b) = (half(y), half(y + 1));
        world.wall_line((cx - a - 2, y), (cx - b - 2, y + 1));
        world.wall_line((cx + a, y), (cx + b, y + 1));
    }
    world.fill_rect(
        cx - half(top) - 2,
        top - 1,
        cx + half(top) + 1,
        top - 1,
        Material::Wall,
    );
    world.fill_rect(
        cx - half(bottom) - 2,
        bottom + 1,
        cx + half(bottom) + 1,
        bottom + 1,
        Material::Wall,
    );
    // Sand in the upper bulb, from a third of the way down to the neck.
    for y in top + (neck - top) / 3..neck {
        for x in cx - half(y)..cx + half(y) {
            world.set(x, y, Material::Sand);
        }
    }
}

fn dam(world: &mut World) {
    let (w, h) = (world.width, world.height);
    let dam = w * 3 / 10;
    let crest = h * 3 / 10;
    world.fill_rect(dam, crest, dam + 3, h - 1, Material::Wall);
    world.fill_rect(0, crest + 4, dam - 1, h - 1, Material::Water);
    // A sandy valley below the dam: a long mound no steeper than sand stands.
    let peak = (w * 7 / 10, h * 3 / 4);
    for x in dam + 4..w {
        // Grass on the sand, starting well clear of the dam.
        let grass = x > dam + 30 && x % 9 < 6;
        let ground = peak.1 + x.abs_diff(peak.0) / 3;
        if ground < h {
            world.fill_rect(x, ground, x, h - 1, Material::Sand);
            if grass {
                world.set(x, ground - 1, Material::Plant);
            }
        }
    }
    // A boulder.
    world.fill_rect(w * 9 / 20, h - 6, w * 9 / 20 + 5, h - 1, Material::Wall);
}

fn wood_fire(world: &mut World) {
    let (w, h) = (world.width, world.height);
    let ground = h - h / 8;
    world.fill_rect(0, ground, w - 1, h - 1, Material::Sand);
    // A pond on the right, with walls for banks.
    let pond = (w * 3 / 4, w - 1);
    world.fill_rect(pond.0, ground - 1, pond.1, h - 1, Material::Wall);
    world.fill_rect(pond.0 + 2, ground - 1, pond.1, h - 3, Material::Water);
    // Trees: trunks and round crowns, on a strip of grass joining them.
    world.fill_rect(0, ground - 1, pond.0 - 1, ground - 1, Material::Plant);
    let spacing = (w / 12).max(8);
    let mut x = spacing / 2;
    let mut k = 0;
    while x + 4 < pond.0 {
        let tall = h / 5 + (k * 7 % 5) * h / 30;
        world.fill_rect(x, ground - tall, x + 1, ground - 1, Material::Plant);
        let r = spacing * 2 / 5 + k % 3;
        let crown = (x, ground - tall);
        world.paint(crown.0, crown.1, r, Material::Plant);
        x += spacing;
        k += 1;
    }
    // The fire, at the foot of the first tree.
    world.paint(spacing / 2 - 2, ground - 2, 1, Material::Fire);
}

/// The U-tube's walls and arms: (left arm's x range, right arm's x range, top row of the
/// arms, bottom row of the channel joining them).
pub fn u_tube_shape(w: usize, h: usize) -> ((usize, usize), (usize, usize), usize, usize) {
    let cx = w / 2;
    let arm = (w / 16).max(4);
    let gap = (w / 10).max(4);
    let left = (cx - gap - arm, cx - gap - 1);
    let right = (cx + gap, cx + gap + arm - 1);
    (left, right, h / 8, h - 4)
}

fn u_tube(world: &mut World) {
    let (w, h) = (world.width, world.height);
    let (left, right, top, bottom) = u_tube_shape(w, h);
    let channel = (bottom - top) / 6;
    world.fill_rect(left.0 - 2, top, right.1 + 2, bottom + 2, Material::Wall);
    world.fill_rect(left.0, top, left.1, bottom, Material::Empty);
    world.fill_rect(right.0, top, right.1, bottom, Material::Empty);
    world.fill_rect(left.0, bottom - channel, right.1, bottom, Material::Empty);
    world.fill_rect(
        left.0,
        top + (bottom - top) / 5,
        left.1,
        bottom,
        Material::Water,
    );
    world.fill_rect(
        left.1 + 1,
        bottom - channel,
        right.1,
        bottom,
        Material::Water,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use Material::*;

    /// Where the cells of `m` are, in scan order.
    fn cells_of(world: &World, m: Material) -> Vec<(usize, usize)> {
        let w = world.width;
        (0..world.cells().len())
            .filter(|&i| material(world.cells()[i]) == m)
            .map(|i| (i % w, i / w))
            .collect()
    }

    fn run(world: &mut World, steps: usize) {
        for _ in 0..steps {
            world.step();
        }
    }

    /// Steps until one moves nothing; panics if that takes more than `most`.
    fn settle(world: &mut World, most: usize) {
        for _ in 0..most {
            world.step();
            if world.changed == 0 {
                return;
            }
        }
        panic!("still moving after {most} steps");
    }

    /// Hand trace: a lone grain falls one cell a step, then rests on the floor.
    #[test]
    fn a_grain_falls_one_cell_a_step_and_lands() {
        let mut world = World::new(5, 5, 1);
        world.set(2, 0, Sand);
        for y in 1..5 {
            world.step();
            assert_eq!(cells_of(&world, Sand), vec![(2, y)]);
        }
        run(&mut world, 3);
        assert_eq!(cells_of(&world, Sand), vec![(2, 4)]);
    }

    /// Hand trace: a grain on top of another slides off to one side or the other, each
    /// about half the time.
    #[test]
    fn a_grain_on_a_grain_slides_off_to_either_side() {
        let mut left = 0;
        for seed in 0..200 {
            let mut world = World::new(3, 2, seed);
            world.set(1, 0, Sand);
            world.set(1, 1, Sand);
            world.step();
            let moved = cells_of(&world, Sand);
            assert!(moved == vec![(0, 1), (1, 1)] || moved == vec![(1, 1), (2, 1)]);
            left += usize::from(moved[0] == (0, 1));
        }
        assert!((80..120).contains(&left), "{left} of 200 went left");
    }

    /// Hand trace: a drop of water on a single wall cell runs off it diagonally, to each
    /// side about half the time, as a grain of sand does.
    #[test]
    fn a_drop_on_a_post_runs_off_to_either_side() {
        let mut left = 0;
        for seed in 0..200 {
            let mut world = World::new(3, 2, seed);
            world.set(1, 1, Wall);
            world.set(1, 0, Water);
            world.step();
            let moved = cells_of(&world, Water);
            assert!(moved == vec![(0, 1)] || moved == vec![(2, 1)], "{moved:?}");
            left += usize::from(moved[0] == (0, 1));
        }
        assert!((80..120).contains(&left), "{left} of 200 went left");
    }

    /// Hand trace: water with nothing under it but the floor moves one cell sideways.
    #[test]
    fn water_on_a_floor_spreads_sideways() {
        let mut world = World::new(5, 1, 3);
        world.set(2, 0, Water);
        world.step();
        let at = cells_of(&world, Water);
        assert!(at == vec![(1, 0)] || at == vec![(3, 0)], "{at:?}");
    }

    /// Sand is heavier than water: it sinks, and the water rises in its place.
    #[test]
    fn sand_sinks_through_water() {
        let mut world = World::new(1, 2, 1);
        world.set(0, 0, Sand);
        world.set(0, 1, Water);
        world.step();
        assert_eq!(world.get(0, 0), Water);
        assert_eq!(world.get(0, 1), Sand);
    }

    /// A wall one cell thick along a diagonal touches itself only at corners. Neither sand
    /// nor water may slip through those corners.
    #[test]
    fn nothing_slips_between_diagonal_walls() {
        let n = 30;
        // The wall runs down to the left, then (mirrored) down to the right.
        for mirror in [false, true] {
            let flip = |x: usize| if mirror { n - 1 - x } else { x };
            let mut world = World::new(n, n, 5);
            for x in 0..=20 {
                world.set(flip(x), 20 - x, Wall);
            }
            let mut rng = Rng::new(9);
            for y in 0..n {
                for x in 0..n {
                    if x + y < 20 && rng.chance(0.6) {
                        world.set(flip(x), y, if rng.chance(0.5) { Sand } else { Water });
                    }
                }
            }
            for _ in 0..500 {
                world.step();
                for m in [Sand, Water] {
                    let inside = |&(x, y): &(usize, usize)| flip(x) + y < 20;
                    assert!(cells_of(&world, m).iter().all(inside), "mirror {mirror}");
                }
            }
        }
    }

    /// A random world of walls, sand and water: however long it runs, not a grain of sand
    /// nor a drop of water is lost or made.
    #[test]
    fn sand_and_water_are_conserved() {
        let mut world = World::new(80, 60, 11);
        let mut rng = Rng::new(4);
        for y in 0..60 {
            for x in 0..80 {
                let r = rng.next_f64();
                let m = match r {
                    r if r < 0.1 => Wall,
                    r if r < 0.35 => Sand,
                    r if r < 0.6 => Water,
                    _ => continue,
                };
                world.set(x, y, m);
            }
        }
        let start = world.counts();
        for _ in 0..30 {
            run(&mut world, 100);
            assert_eq!(world.counts(), start);
        }
    }

    /// Plants grow by turning water into plant, one cell for one cell, so with nothing
    /// burning, water plus plant stays the same while plants spread.
    #[test]
    fn plants_grow_by_drinking_water() {
        let mut world = World::new(40, 30, 2);
        world.fill_rect(0, 10, 39, 29, Water);
        world.set(20, 20, Plant);
        // Growing only up and out, the plant leaves the water below it.
        let below = |world: &World| (21..30).all(|y| world.get(20, y) == Water);
        let start = world.counts();
        run(&mut world, 2_000);
        let now = world.counts();
        assert!(now[Plant as usize] > 50, "{now:?}");
        assert!(below(&world));
        assert_eq!(
            now[Plant as usize] + now[Water as usize],
            start[Plant as usize] + start[Water as usize]
        );
        assert_eq!(now[Sand as usize], start[Sand as usize]);
    }

    /// The angle of repose. A grain rests only when the cells below it and diagonally below
    /// are full, so in a pile at rest no column stands more than one cell above the column
    /// beside it: a slope of 45 degrees at most. Poured from one point, the pile is a
    /// triangle that steep, so N grains stand about sqrt(N) high; a rule that let sand
    /// slide further would make it lower.
    #[test]
    fn a_pile_stands_at_45_degrees() {
        let (w, h, grains) = (121, 60, 900);
        let mut world = World::new(w, h, 7);
        for k in 0..grains * 2 {
            if k % 2 == 0 {
                world.set(w / 2, 0, Sand);
            }
            world.step();
        }
        settle(&mut world, 1_000);
        assert_eq!(world.count(Sand), grains);
        let heights: Vec<usize> = (0..w)
            .map(|x| (0..h).filter(|&y| world.get(x, y) == Sand).count())
            .collect();
        // Columns are solid: no grain rests on an empty cell.
        for (x, height) in heights.iter().enumerate() {
            assert!(
                (h - height..h).all(|y| world.get(x, y) == Sand),
                "column {x}"
            );
        }
        for x in 1..w {
            assert!(heights[x].abs_diff(heights[x - 1]) <= 1, "{heights:?}");
        }
        let peak = *heights.iter().max().unwrap();
        assert!(peak >= 28, "peak {peak}, expected about 30: {heights:?}");
    }

    /// The level in an arm of the U-tube: the top row that is water all the way across.
    fn level(world: &World, arm: (usize, usize), top: usize, bottom: usize) -> usize {
        (top..=bottom)
            .find(|&y| (arm.0..=arm.1).all(|x| world.get(x, y) == Water))
            .unwrap_or(bottom + 1)
    }

    /// Communicating vessels: water poured into one arm of a U-tube rises in the other
    /// until the two levels agree (within a row). Without the pressure rule it doesn't.
    #[test]
    fn water_levels_out_in_a_u_tube() {
        let (w, h) = (160, 100);
        let (left, right, top, bottom) = u_tube_shape(w, h);
        for pressure in [true, false] {
            let mut world = World::new(w, h, 3);
            world.pressure = pressure;
            u_tube(&mut world);
            let water = world.count(Water);
            let before = (
                level(&world, left, top, bottom),
                level(&world, right, top, bottom),
            );
            assert!(before.1 > before.0 + 20, "{before:?}");
            run(&mut world, 3_000);
            assert_eq!(world.count(Water), water);
            let after = (
                level(&world, left, top, bottom),
                level(&world, right, top, bottom),
            );
            if pressure {
                assert!(after.0.abs_diff(after.1) <= 1, "{before:?} to {after:?}");
            } else {
                assert!(after.1 > after.0 + 20, "{before:?} to {after:?}");
            }
        }
    }

    /// Fire burns through every plant joined to where it was lit, across edges or corners,
    /// and then goes out. A plant below the fire, not joined to it, survives: flames only
    /// rise.
    #[test]
    fn fire_burns_every_joined_plant_then_goes_out() {
        let mut world = World::new(60, 50, 8);
        // Three blobs joined only corner to corner.
        world.fill_rect(5, 10, 14, 19, Plant);
        world.set(15, 20, Plant);
        world.fill_rect(16, 21, 25, 30, Plant);
        world.set(26, 31, Plant);
        world.fill_rect(27, 32, 36, 36, Plant);
        // Out of reach, below.
        world.fill_rect(5, 42, 50, 46, Plant);
        let spared = cells_of(&world, Plant)
            .into_iter()
            .filter(|&(_, y)| y >= 42)
            .collect::<Vec<_>>();
        world.set(5, 10, Fire);
        let mut burning = 0;
        for _ in 0..2_000 {
            world.step();
            burning = burning.max(world.count(Fire));
        }
        assert!(burning > 50);
        assert_eq!(world.count(Fire), 0);
        assert_eq!(cells_of(&world, Plant), spared);
    }

    /// Water puts a flame out, and isn't used up doing it.
    #[test]
    fn water_puts_fire_out() {
        let mut world = World::new(3, 3, 1);
        world.fill_rect(0, 2, 2, 2, Wall);
        world.set(1, 1, Fire);
        world.set(0, 1, Water);
        world.step();
        assert_eq!(world.count(Fire), 0);
        assert_eq!(world.count(Water), 1);
    }

    /// A lone flame rises a cell each step (straight up or to one side) and burns out
    /// within its life of at most 16 steps.
    #[test]
    fn a_flame_rises_and_dies() {
        for seed in 0..20 {
            let mut world = World::new(40, 40, seed);
            world.set(20, 30, Fire);
            world.step();
            let at = cells_of(&world, Fire);
            assert_eq!(at.len(), 1);
            assert_eq!(at[0].1, 29);
            assert!(at[0].0.abs_diff(20) <= 1);
            run(&mut world, 15);
            assert_eq!(world.count(Fire), 0, "seed {seed}");
        }
    }

    /// A z-score for "no side is favoured": the sum of the runs' offsets over the root of
    /// the sum of their squares, which for unbiased runs is about normal with mean 0 and
    /// standard deviation 1.
    fn z(offsets: &[f64]) -> f64 {
        let sum: f64 = offsets.iter().sum();
        let squares: f64 = offsets.iter().map(|d| d * d).sum();
        sum / squares.sqrt()
    }

    /// How far `world`'s cells of `m` lie to the right of column `centre`, in all.
    fn offset(world: &World, m: Material, centre: f64) -> f64 {
        cells_of(world, m)
            .iter()
            .map(|&(x, _)| x as f64 - centre)
            .sum()
    }

    /// No side is favoured. Over many runs, each symmetrical:
    /// - sand poured on one point piles up as much to its left as to its right;
    /// - so does water poured on one point, where every drop that lands on the puddle
    ///   chooses a side;
    /// - water in the middle arm of a tube with three arms rises as much in the left arm as
    ///   in the right, which tests the pressure rule.
    ///
    /// Each run's offset of its centre of mass is noise around 0.
    #[test]
    fn no_side_is_favoured() {
        let (mut sand, mut water, mut arms) = (Vec::new(), Vec::new(), Vec::new());
        for seed in 0..40 {
            let (w, h) = (121, 40);
            for (m, out) in [(Sand, &mut sand), (Water, &mut water)] {
                let mut world = World::new(w, h, seed);
                for k in 0..600 {
                    if k % 2 == 0 {
                        world.set(w / 2, 0, m);
                    }
                    world.step();
                }
                if m == Sand {
                    settle(&mut world, 1_000);
                } else {
                    run(&mut world, 200); // water never quite settles
                }
                out.push(offset(&world, m, w as f64 / 2.0 - 0.5));
            }

            // Arms 10 wide, 4 apart, the middle one full and the others empty: symmetrical
            // about x = 21.5.
            let mut world = World::new(48, 40, seed);
            world.fill_rect(0, 0, 47, 39, Wall);
            for left in [3, 17, 31] {
                world.fill_rect(left, 2, left + 9, 37, Empty);
            }
            world.fill_rect(3, 33, 40, 37, Water);
            world.fill_rect(17, 5, 26, 32, Water);
            run(&mut world, 3);
            arms.push(offset(&world, Water, 21.5));
        }
        let (zs, zw, za) = (z(&sand), z(&water), z(&arms));
        assert!(
            zs.abs() < 4.0 && zw.abs() < 4.0 && za.abs() < 4.0,
            "z: sand {zs:.2}, water {zw:.2}, three arms {za:.2}"
        );
    }

    /// Water whose levels are one row apart is level as far as the pressure rule goes, and
    /// a tube of full rows stays exactly as it is: nothing in it can move.
    #[test]
    fn levels_one_row_apart_are_at_rest() {
        let mut world = World::new(30, 20, 1);
        world.fill_rect(0, 0, 29, 19, Wall);
        world.fill_rect(2, 2, 9, 17, Empty);
        world.fill_rect(20, 2, 27, 17, Empty);
        world.fill_rect(2, 14, 27, 17, Empty);
        world.fill_rect(2, 8, 9, 17, Water);
        world.fill_rect(10, 14, 27, 17, Water);
        world.fill_rect(20, 9, 27, 13, Water);
        let start = world.cells().iter().map(|&c| c & KIND).collect::<Vec<_>>();
        for _ in 0..20 {
            world.step();
            assert_eq!(world.changed, 0);
        }
        let end = world.cells().iter().map(|&c| c & KIND).collect::<Vec<_>>();
        assert_eq!(start, end);
    }

    /// The hourglass runs out: after long enough every grain is in the lower bulb, and
    /// none is lost on the way.
    #[test]
    fn the_hourglass_runs_out() {
        let mut world = World::new(160, 100, 1);
        hourglass(&mut world);
        let grains = world.count(Sand);
        assert!(grains > 500, "{grains}");
        let (_, neck, _, _) = hourglass_shape(100);
        run(&mut world, 4_000);
        assert_eq!(world.count(Sand), grains);
        assert!(cells_of(&world, Sand).iter().all(|&(_, y)| y > neck));
    }

    /// A line of plant along the bottom of a pond grows up through it and drinks it dry.
    #[test]
    fn plants_drink_a_pond_dry() {
        let mut world = World::new(40, 30, 2);
        world.fill_rect(0, 10, 39, 28, Water);
        world.fill_rect(0, 29, 39, 29, Plant);
        let total = world.count(Water) + world.count(Plant);
        run(&mut world, 10_000);
        assert_eq!(world.count(Water), 0);
        assert_eq!(world.count(Plant), total);
    }

    /// The fire opening's fire burns through the whole wood, every tree joined by the
    /// grass, and then goes out.
    #[test]
    fn the_fire_opening_burns_the_whole_wood() {
        let mut world = World::new(320, 200, 1);
        wood_fire(&mut world);
        let (water, sand) = (world.count(Water), world.count(Sand));
        assert!(world.count(Plant) > 2_000);
        run(&mut world, 3_000);
        assert_eq!(world.count(Plant), 0);
        assert_eq!(world.count(Fire), 0);
        assert_eq!((world.count(Water), world.count(Sand)), (water, sand));
    }

    /// In the fire opening, a puddle of water on the grass between two trees is a
    /// firebreak: every plant beyond it survives (and some grow, drinking the puddle), and
    /// the wood before it burns.
    #[test]
    fn a_puddle_on_the_grass_stops_the_fire() {
        let (w, h) = (320, 200);
        let mut world = World::new(w, h, 1);
        wood_fire(&mut world);
        let ground = h - h / 8;
        let spacing = w / 12;
        let x = spacing / 2 + spacing * 2 + spacing / 2; // between the third and fourth trees
        for _ in 0..3 {
            world.paint(x, ground - 6, 3, Water);
        }
        let beyond = |world: &World| {
            cells_of(world, Plant)
                .iter()
                .filter(|&&(px, _)| px > x)
                .count()
        };
        let before = beyond(&world);
        run(&mut world, 3_000);
        assert_eq!(world.count(Fire), 0);
        assert!(beyond(&world) >= before);
        // The wood before the puddle burns, all but the plants under the puddle, wet. The
        // puddle has spread over the grass, so find its left end.
        let puddle = cells_of(&world, Water)
            .iter()
            .map(|&(px, _)| px)
            .min()
            .unwrap();
        assert!(
            cells_of(&world, Plant)
                .iter()
                .all(|&(px, _)| px + 1 >= puddle)
        );
    }

    /// Breaking the dam lets the lake into the valley, and its level falls; the grass
    /// there drinks some of it.
    #[test]
    fn breaking_the_dam_floods_the_valley() {
        let (w, h) = (320, 200);
        let mut world = World::new(w, h, 1);
        dam(&mut world);
        let start = world.counts();
        let dam_x = w * 3 / 10;
        let below = |world: &World| {
            cells_of(world, Water)
                .iter()
                .filter(|&&(x, _)| x > dam_x + 3)
                .count()
        };
        run(&mut world, 500);
        assert_eq!(below(&world), 0, "the dam holds until broken");
        world.fill_rect(dam_x, h - 20, dam_x + 3, h - 1, Empty);
        run(&mut world, 3_000);
        let now = world.counts();
        assert!(
            below(&world) > start[Water as usize] / 10,
            "{}",
            below(&world)
        );
        assert!(now[Plant as usize] > start[Plant as usize]);
        // The lake and the flood beside the dam are one body, and level.
        let surface = |x: usize| (0..h).find(|&y| world.get(x, y) == Water).unwrap();
        let (lake, flood) = (surface(10), surface(dam_x + 8));
        assert!(lake.abs_diff(flood) <= 1, "lake {lake}, flood {flood}");
        assert_eq!(
            now[Water as usize] + now[Plant as usize],
            start[Water as usize] + start[Plant as usize]
        );
    }

    /// Sand poured into one arm of the U-tube sinks, and the water it displaces rises in
    /// both arms, which stay level.
    #[test]
    fn sand_in_a_u_tube_raises_both_levels() {
        let (w, h) = (160, 100);
        let (left, right, top, bottom) = u_tube_shape(w, h);
        let mut world = World::new(w, h, 2);
        u_tube(&mut world);
        run(&mut world, 1_000);
        let before = (
            level(&world, left, top, bottom),
            level(&world, right, top, bottom),
        );
        for k in 0..400 {
            if k % 2 == 0 {
                world.set((left.0 + left.1) / 2, top, Sand);
            }
            world.step();
        }
        run(&mut world, 1_000);
        let after = (
            level(&world, left, top, bottom),
            level(&world, right, top, bottom),
        );
        assert!(
            after.0 < before.0 && after.1 < before.1,
            "{before:?} to {after:?}"
        );
        assert!(after.0.abs_diff(after.1) <= 1, "{after:?}");
    }

    /// Painting fills only empty cells, except that fire also lights plants and erasing
    /// clears anything.
    #[test]
    fn painting_respects_what_is_there() {
        let mut world = World::new(5, 1, 1);
        world.set(1, 0, Wall);
        world.set(3, 0, Plant);
        world.paint(2, 0, 2, Sand);
        assert_eq!(world.get(1, 0), Wall);
        assert_eq!(world.get(3, 0), Plant);
        assert_eq!(world.count(Sand), 3);
        world.paint(3, 0, 0, Fire);
        assert_eq!(world.get(3, 0), Fire);
        world.paint(2, 0, 2, Empty);
        assert_eq!(world.count(Empty), 5);
    }
}

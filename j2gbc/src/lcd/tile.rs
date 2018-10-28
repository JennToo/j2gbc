const TILE_SIZE: usize = 8;
#[cfg(test)]
const TILE_2BPP_LEN: usize = 16;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct MonoTile {
    data: [[u8; TILE_SIZE]; TILE_SIZE],
}

pub type MonoTileRow = [u8; TILE_SIZE];

impl MonoTile {
    #[cfg(test)]
    pub fn from_2bpp(data: &[u8]) -> MonoTile {
        assert_eq!(data.len(), TILE_2BPP_LEN);
        let mut t = MonoTile::default();

        for row in 0..TILE_SIZE {
            t.update_row(row, data[row * 2], data[row * 2 + 1]);
        }

        t
    }

    pub fn update_row(&mut self, row: usize, lo: u8, hi: u8) {
        for i in 0..TILE_SIZE {
            self.data[row][i] = read_bit(lo, (TILE_SIZE - 1 - i) as u8)
                | (read_bit(hi, (TILE_SIZE - 1 - i) as u8) << 1);
        }
    }

    pub fn read_row(&self, row: usize) -> MonoTileRow {
        self.data[row]
    }
}

#[test]
fn test_tile_read() {
    let t = MonoTile::from_2bpp(&[
        0b0000_0001,
        0b0000_0001,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b0000_0000,
        0b1000_0000,
        0b0000_0001,
        0b0000_0000,
    ]);

    for x in 0..TILE_SIZE {
        for y in 0..TILE_SIZE {
            let d = t.data[y][x];
            if x == 7 && y == 0 {
                assert_eq!(d, 3);
            } else if x == 0 && y == 6 {
                assert_eq!(d, 2);
            } else if x == 7 && y == 7 {
                assert_eq!(d, 1);
            } else {
                assert_eq!(d, 0);
            }
        }
    }
}

fn read_bit(value: u8, bit: u8) -> u8 {
    let mask = 1 << bit;
    (value & mask) >> bit
}

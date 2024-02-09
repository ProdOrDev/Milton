//! Emulation of the 16x16 LCD pixel matrix display.

use crate::hughes0488::{Column, Row};
use std::num::NonZeroU8;

/// The status of an LCD pixel.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pixel {
    /// This pixel is currently on (enabled).
    On {
        /// The amount of fade this pixel has.
        ///
        /// This is a decrementing counter, so once it reaches zero or
        /// the specified maximum fade level the pixel will be disabled.
        fade: NonZeroU8,
    },
    /// This pixel is currently off (disabled).
    #[default]
    Off,
}

/// A 16x16 LCD pixel matrix display.
#[derive(Default, Debug, Clone)]
pub struct Lcd {
    /// The raw pixel data.
    pub pixels: [[Pixel; 16]; 16],
    /// A flag indicating if the display has been updated.
    dirty: bool,
}

impl Lcd {
    /// Update this LCD display.
    pub fn clock(&mut self, row: Row, col: Column) {
        // If all the row indexes or the column data are zero, nothing will
        // be updated.
        if row.0 == 0 || col.0 == 0 {
            return;
        }

        for y in 0..=15 {
            // If the current row index is not set, nothing will be updated.
            if row.0 >> y & 1 == 0 {
                continue;
            }

            for x in 0..=15 {
                // Pixels are not set/unset through the row/column lines,
                // instead they are enabled and eventually decay to off
                // over a brief period of time.
                if col.0 >> x & 1 != 0 {
                    let old = self.pixels[y][x];
                    let new = Pixel::On {
                        fade: NonZeroU8::MAX,
                    };

                    self.dirty |= old != new;
                    self.pixels[y][x] = new;
                }
            }
        }
    }

    /// Check if the LCD display has been updated.
    ///
    /// This can allow certain renderers to avoid redrawing/recalculating
    /// the frame if the LCD panel has not changed.
    #[must_use]
    pub fn is_dirty(&mut self) -> bool {
        std::mem::replace(&mut self.dirty, false)
    }

    /// Decay all the pixel values by a single fade level.
    ///
    /// If the a pixel has reached zero or gone above the maximum specified
    /// fade level, the pixel will be disabled.
    pub fn decay(&mut self, maximum_fade: u8) {
        for row in self.pixels.iter_mut() {
            for col in row {
                *col = match col {
                    Pixel::On { fade } => {
                        self.dirty = true;

                        let fade = fade.get().saturating_sub(1);

                        if fade == 0 || fade < (255 - maximum_fade) {
                            Pixel::Off
                        } else {
                            Pixel::On {
                                fade: NonZeroU8::new(fade).unwrap(),
                            }
                        }
                    }
                    Pixel::Off => Pixel::Off,
                }
            }
        }
    }
}

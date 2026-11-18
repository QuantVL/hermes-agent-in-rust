//! Animated pet overlay state — pure frame/step state, no display.
//!
//! Advancing the pet is deterministic arithmetic with wrap-around at
//! [`PET_FRAME_COUNT`]. No GPUI window, animation timer or gateway connection
//! is involved, so the exact frame/step progression can be asserted in a
//! headless test.

use hermes_gpui::WindowType;

/// Number of frames in the pet's idle animation loop.
pub const PET_FRAME_COUNT: usize = 4;

/// Frame/step state for the animated pet overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PetOverlay {
    /// Current animation frame in `0..PET_FRAME_COUNT`.
    pub frame: usize,
    /// Monotonic advance counter (number of steps taken).
    pub step: usize,
    /// Canonical window kind for this overlay.
    pub window: WindowType,
}

impl PetOverlay {
    /// Build a pet at frame 0, step 0.
    pub fn new() -> Self {
        Self {
            frame: 0,
            step: 0,
            window: WindowType::PetOverlay,
        }
    }

    /// The number of frames in one idle loop.
    pub fn frame_count(&self) -> usize {
        PET_FRAME_COUNT
    }

    /// PURE: the next `(frame, step)` with wrap-around at [`PET_FRAME_COUNT`].
    ///
    /// Takes plain numbers and returns plain numbers; no state is mutated and
    /// no display or timer is touched.
    pub fn next_state(frame: usize, step: usize) -> (usize, usize) {
        ((frame + 1) % PET_FRAME_COUNT, step + 1)
    }

    /// PURE: a new overlay advanced by one deterministic step.
    pub fn next(&self) -> Self {
        let (frame, step) = Self::next_state(self.frame, self.step);
        Self {
            frame,
            step,
            window: self.window,
        }
    }

    /// Advance this overlay by one deterministic step.
    pub fn advance(&mut self) {
        let (frame, step) = Self::next_state(self.frame, self.step);
        self.frame = frame;
        self.step = step;
    }

    /// Reset the animation to frame 0, step 0.
    pub fn reset(&mut self) {
        self.frame = 0;
        self.step = 0;
    }
}

impl Default for PetOverlay {
    fn default() -> Self {
        Self::new()
    }
}

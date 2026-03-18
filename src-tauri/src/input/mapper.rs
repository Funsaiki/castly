/// Maps viewport coordinates to phone screen coordinates.
///
/// The viewport maintains the phone's aspect ratio with letterboxing,
/// so coordinates need to be transformed from the viewport space to
/// the phone's actual pixel coordinates.
#[derive(Debug, Clone)]
pub struct CoordinateMapper {
    phone_width: u32,
    phone_height: u32,
    viewport_width: f64,
    viewport_height: f64,
    offset_x: f64,
    offset_y: f64,
    scale: f64,
}

impl CoordinateMapper {
    pub fn new(phone_width: u32, phone_height: u32) -> Self {
        Self {
            phone_width,
            phone_height,
            viewport_width: phone_width as f64,
            viewport_height: phone_height as f64,
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
        }
    }

    /// Update when the viewport resizes
    pub fn update_viewport(&mut self, viewport_width: f64, viewport_height: f64) {
        self.viewport_width = viewport_width;
        self.viewport_height = viewport_height;

        let phone_aspect = self.phone_width as f64 / self.phone_height as f64;
        let viewport_aspect = viewport_width / viewport_height;

        if viewport_aspect > phone_aspect {
            // Viewport is wider - pillarbox (black bars on sides)
            self.scale = viewport_height / self.phone_height as f64;
            let content_width = self.phone_width as f64 * self.scale;
            self.offset_x = (viewport_width - content_width) / 2.0;
            self.offset_y = 0.0;
        } else {
            // Viewport is taller - letterbox (black bars top/bottom)
            self.scale = viewport_width / self.phone_width as f64;
            let content_height = self.phone_height as f64 * self.scale;
            self.offset_x = 0.0;
            self.offset_y = (viewport_height - content_height) / 2.0;
        }
    }

    /// Convert viewport coordinates to phone coordinates
    pub fn viewport_to_phone(&self, viewport_x: f64, viewport_y: f64) -> Option<(f32, f32)> {
        let phone_x = (viewport_x - self.offset_x) / self.scale;
        let phone_y = (viewport_y - self.offset_y) / self.scale;

        // Check bounds
        if phone_x < 0.0
            || phone_y < 0.0
            || phone_x >= self.phone_width as f64
            || phone_y >= self.phone_height as f64
        {
            return None; // Click is outside the phone screen area
        }

        Some((phone_x as f32, phone_y as f32))
    }

    pub fn phone_width(&self) -> u32 {
        self.phone_width
    }

    pub fn phone_height(&self) -> u32 {
        self.phone_height
    }
}

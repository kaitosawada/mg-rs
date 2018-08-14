extern crate find_folder;

use conrod;
use conrod::{widget, Colorable, Positionable, Widget};
use std;
use gfx;
use gfx_device_gl;
use piston_window::{UpdateEvent, Window, self, generic_event};


widget_ids!(struct Ids { text });

pub struct UIHandler<'a> {
    ui: conrod::Ui,
    ids: Ids,
    image_map: conrod::image::Map<piston_window::Texture<gfx_device_gl::Resources>>,
    text_texture_cache: piston_window::Texture<gfx_device_gl::Resources>,
    glyph_cache: conrod::text::GlyphCache<'a>,
}

impl<'a> UIHandler<'a> {
    pub fn new(window: &mut piston_window::PistonWindow, width: u32, height: u32) -> Self {
        //setting the ui
        let mut ui = conrod::UiBuilder::new([width as f64, height as f64])
            .theme(theme())
            .build();

        let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("assets").unwrap();
        let font_path = assets.join("fonts/NotoSans/NotoSans-Regular.ttf");
        ui.fonts.insert_from_file(font_path).unwrap();

        let ids = Ids::new(ui.widget_id_generator());

        //making renderer using draw ui
        let image_map = conrod::image::Map::new();

        let (mut glyph_cache, mut text_texture_cache) = {
            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;
            let cache = conrod::text::GlyphCache::new(width, height, SCALE_TOLERANCE, POSITION_TOLERANCE);
            let buffer_len = width as usize * height as usize;
            let init = vec![128; buffer_len];
            let settings = piston_window::TextureSettings::new();
            let factory = &mut window.factory;
            let texture = piston_window::G2dTexture::from_memory_alpha(factory, &init, width, height, &settings).unwrap();
            (cache, texture)
        };


        UIHandler {
            ui,
            ids,
            image_map,
            text_texture_cache,
            glyph_cache,
        }
    }

    pub fn update(&mut self, window: &mut piston_window::PistonWindow, event: &piston_window::Event) {
        let size = window.size();
        let (win_w, win_h) = (size.width as conrod::Scalar, size.height as conrod::Scalar);
        if let Some(e) = conrod::backend::piston::event::convert(event.clone(), win_w, win_h) {
            self.ui.handle_event(e);
        }
        event.update(|_| {
            let ui = &mut self.ui.set_widgets();

            // "Hello World!" in the middle of the screen.
            widget::Text::new("aaa")//&format!("{:.*}", 2, 1.0 / diff))
                .middle_of(ui.window)
                .color(conrod::color::WHITE)
                .font_size(16)
                .set(self.ids.text, ui);
        });

        //
        window.draw_2d(event, |context, graphics| {
            if let Some(primitives) = self.ui.draw_if_changed() {
                let cache_queued_glyphs =
                    |graphics: &mut piston_window::G2d,
                     cache: &mut piston_window::G2dTexture,
                     rect: conrod::text::rt::Rect<u32>,
                     data: &[u8]| {
                        let mut text_vertex_data = Vec::new();
                        let offset = [rect.min.x, rect.min.y];
                        let size = [rect.width(), rect.height()];
                        let format = piston_window::texture::Format::Rgba8;
                        let encoder = &mut graphics.encoder;
                        text_vertex_data.clear();
                        text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                        piston_window::texture::UpdateTexture::update(cache, encoder, format, &text_vertex_data[..], offset, size)
                            .expect("failed to update texture")
                    };
                const SCALE_TOLERANCE: f32 = 0.1;
                const POSITION_TOLERANCE: f32 = 0.1;
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston::draw::primitives(
                    primitives,
                    context,
                    graphics,
                    &mut self.text_texture_cache,
                    &mut self.glyph_cache,
                    &self.image_map,
                    cache_queued_glyphs,
                    texture_from_image);
            }
        });
    }

    pub fn draw(&mut self) {}
}

pub fn theme() -> conrod::Theme {
    use self::conrod::position::{Align, Direction, Padding, Position, Relative};
    conrod::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod::color::DARK_CHARCOAL,
        shape_color: conrod::color::LIGHT_CHARCOAL,
        border_color: conrod::color::BLACK,
        border_width: 0.0,
        label_color: conrod::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}
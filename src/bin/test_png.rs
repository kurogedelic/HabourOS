use sdl2::image::LoadSurface;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    // SDL2_imageを初期化
    sdl2::image::init(sdl2::image::InitFlag::PNG)?;

    let window = video
        .window("PNG Test", 480, 320)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let mut events = sdl.event_pump()?;

    println!("Loading PNG: assets/images/buttons/close.png");

    // PNGをロード
    let surface = sdl2::surface::Surface::from_file("assets/images/buttons/close.png")
        .map_err(|e| e.to_string())?;

    println!("Surface loaded: {}x{}", surface.width(), surface.height());

    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    println!("Texture created");

    loop {
        for e in events.poll_iter() {
            match e {
                sdl2::event::Event::Quit { .. }
                | sdl2::event::Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => return Ok(()),
                _ => {}
            }
        }

        // 背景をアンバー色に
        canvas.set_draw_color(Color::RGB(255, 144, 24));
        canvas.clear();

        // PNGを描画（拡大して見やすく）
        canvas.copy(&texture, None, Rect::new(50, 50, 64, 64))?;

        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

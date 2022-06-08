use engine;
use crate::constants as constants;

use winit::{
    event_loop::{ EventLoop, ControlFlow },
    event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent},
    window::{Window, WindowBuilder},
};

pub struct Application { // lifetime <'a>
    window: Window,
    // objects: Vec<String>,
    // camera: &'a String,
    // camera_controler: ???
    // renderer: engine::renderer,
    // render_system: engine::render_system,
}

impl Application {
    pub fn new() -> (EventLoop<()>, Self) {
        let (event_loop, window) = Self::new_window();
        
        (event_loop, Self {
            window,
        })
    }
    
    fn new_window() -> (EventLoop<()>, Window) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(constants::WINDOW_NAME)
            .with_inner_size(winit::dpi::LogicalSize::new(constants::WINDOW_WIDTH, constants::WINDOW_HEIGHT))
            .with_resizable(true)
            .build(&event_loop)
            .expect("Failed to create window.");

        (event_loop, window)
    }

    fn draw_frame(&mut self) {
        engine::hello();
        // TODO: Drawing logic
        // TODO: Limit draw calls per second (FPS)
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow|
            match event {
                //? Events from the window
                Event::WindowEvent { ref event, ..} => {
                        match event { 
                            // Check the keyboard inputs or window for a closing request
                            // Its matching for either struct (with the following contents) 
                            WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                                input: KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => *control_flow = ControlFlow::Exit,
    
                            // Unssuported Window Events
                            _ => {}
                        }
                },
    
                //? Events for rendering
                Event::RedrawRequested(_window_id) =>  {
                    self.draw_frame();
                },
    
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually request it.
                    self.window.request_redraw();
                },
                
                //? The rest of events
                _ => {}
            }
        );
    }
}
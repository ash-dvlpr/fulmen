use crate::constants as constants;
use engine::{
    app::{LveApplication, LveApplicationBuilder}, 
    winit::{
        event_loop::{ EventLoop, ControlFlow },
        event::{Event, VirtualKeyCode, ElementState, KeyboardInput, WindowEvent},
        // window::{Window, WindowBuilder},
    },
};


pub struct Application { // lifetime <'a>
    engine_app: LveApplication, 
    // objects: Vec<String>,
    // camera: &'a String,
    // camera_controler: ???
}

impl Application {
    pub fn new() -> (Self, EventLoop<()>) {
        let (engine_app, event_loop) = LveApplication::builder()
            .with_window_name(constants::WINDOW_NAME)
            .with_resizable_window(false)
            .build();

        (Self{
            engine_app,   
        }, event_loop)
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
    
                            // ! Testing. Calls engine::hello() when pressing 'H'
                            WindowEvent::KeyboardInput { 
                                input: KeyboardInput { 
                                    state: ElementState::Pressed, 
                                    virtual_keycode: Some(VirtualKeyCode::H),
                                    .. 
                                }, .. 
                            } => { engine::hello(); },

                            // Unssuported Window Events
                            _ => {}
                        }
                },
    
                //? Events for rendering
                Event::RedrawRequested(_window_id) =>  {
                    // self.draw_frame();
                },
    
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually request it.
                    self.engine_app.window.request_redraw(); // This should be moved into LveApplication
                },
                
                //? The rest of events
                _ => {}
            }
        );
    }
}
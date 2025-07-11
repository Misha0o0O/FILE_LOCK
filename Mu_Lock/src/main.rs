use smithay::{
    backend::{
        allocator::gbm::GbmAllocator,
        drm::DrmNode,
        renderer::{
            damage::DamageTrackedRenderer,
            element::element_tree::render_elements_list,
            vulkan::{Instance, VulkanBackend, VulkanSurface},
            ImportAll, Renderer,
        },
    },
    desktop::{space::Space, Kind},
    input::{
        keyboard::{KeyEvent, KeyboardHandle, KeyboardTarget},
        pointer::{MotionEvent, PointerHandle, PointerTarget},
        Seat, SeatState,
    },
    output::{Mode, Output, PhysicalProperties},
    reexports::{calloop::EventLoop, wayland_server::Display},
    utils::{Transform, Size},
    wayland::seat::XkbConfig,
};

pub struct CompositorState {
    pub seat: Seat<Kind>,
    pub seat_state: SeatState<Kind>,
    pub pointer: PointerHandle<Kind>,
    pub keyboard: KeyboardHandle<Kind>,
    pub space: Space<Kind>,
    pub output: Output,
    pub pointer_position: (f64, f64),
    pub lock_duration_minutes: u32,
    pub typing_input: String,
    pub input_activbe: bool,
}
 fn draw_lock_duration_input(state: &mut CompositorState) {
    let rect = Rectangle::from_loc_and_size((20,20), (220,40));
    let bg_color = if state.input_active {
        [0.3, 0.3, 0.8, 0.6] // active
    } else {
        [0.2, 0.2, 0.2, 0.5] // inactive
    };

    let input_box = LayerSurface:;new-rectanle(rect, bg_color);
    state.space.add_overlay(&state.output, input_box);
 }

impl PointerTarget<Kind> for CompositorState {
    fn motion(&mut self, _: &Seat<Kind>, event: &MotionEvent<Kind>) {
        self.pointer_position = event.location();
        println!("🖱 Pointer moved to: {:?}", self.pointer_position);
    }

    impl KeyboardTarget<Kind> for CompositorState {
    fn key(&mut self, _: &Seat<Kind>, event: &KeyEvent) {
        if !self.input_active {
            return;
        }

        use smithay::input::keyboard::Keysym;

        match event.keysym {
            Keysym::BackSpace => {
                self.typing_input.pop();
            }
            Keysym::Return => {
                if let Ok(val) = self.typing_input.parse::<u32>() {
                    self.lock_duration_minutes = val;
                    println!("🔒 Set lock duration: {} minutes", val);
                }
                self.typing_input.clear();
                self.input_active = false;
            }
            Keysym::Escape => {
                self.typing_input.clear();
                self.input_active = false;
            }
            _ => {
                if let Some(ch) = event.text.clone() {
                    self.typing_input.push_str(&ch);
                }
            }
        }
    }
}

}

impl KeyboardTarget<Kind> for CompositorState {
    fn key(&mut self, _: &Seat<Kind>, event: &KeyEvent) {
        if !self.input_active {
            return;
        }

        use smithay::input::keyboard::Keysym;

        match event.keysym {
            Keysym::BackSpace => {
                self.typing_input.pop();
            }
            Keysym::Return => {
                if let Ok(val) = self.typing_input.parse::<u32>() {
                    self.lock_duration_minutes = val;
                    println!("🔒 Set lock duration: {} minutes", val);
                }
                self.typing_input.clear();
                self.input_active = false;
            }
            Keysym::Escape => {
                self.typing_input.clear();
                self.input_active = false;
            }
            _ => {
                if let Some(ch) = event.text.clone() {
                    self.typing_input.push_str(&ch);
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut display = Display::new()?;
    let display_handle = display.handle();
    let mut event_loop = EventLoop::try_new()?;

    // Initialize input
    let mut seat_state = SeatState::new();
    let seat = seat_state.new_wl_seat(&display_handle, "seat-0", None);
    seat.add_pointer();
    seat.add_keyboard(XkbConfig::default(), 200, 25)?;

    let pointer = seat.get_pointer().unwrap();
    let keyboard = seat.get_keyboard().unwrap();

    // Vulkan renderer
    let instance = Instance::new()?;
    let (mut backend, _node) = VulkanBackend::new(instance, None)?;
    let mut renderer = backend.renderer()?;

    // Virtual output (640x480)
    let size: Size<i32, _> = (640, 480).into();
    let output = Output::new("virtual-output", PhysicalProperties::default());
    let mode = Mode { size, refresh: 60_000 };
    output.change_current_state(Some(mode), Some(Transform::Normal), None, None);
    output.set_preferred(mode);

    let surface = VulkanSurface::create(&mut backend, size)?;
    let mut damage_renderer = DamageTrackedRenderer::new(renderer.clone());

    // Space setup
    let mut space: Space<Kind> = Space::default();
    space.map_output(output.clone());

    let mut state = CompositorState {
        seat,
        seat_state,
        pointer,
        keyboard,
        space,
        output,
        pointer_position: (0.0, 0.0),
    };

    println!("✅ Compositor running with Vulkan @ 640x480");

    loop {
        display.dispatch_clients(&mut (), std::time::Duration::from_millis(16))?;
        event_loop.dispatch(std::time::Duration::from_millis(16), &mut ())?;

        let elements = render_elements_list::<_, Kind>(&state.space, &state.output);
        damage_renderer.render(&surface, [0.1, 0.1, 0.1, 1.0], &elements, &state.output, 1.0)?;

        draw_lock_duration_input(state)
        display.flush_clients()?;
    }
}

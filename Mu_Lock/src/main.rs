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

struct CompositorState {
    seat: Seat<Kind>,
    seat_state: SeatState<Kind>,
    pointer: PointerHandle<Kind>,
    keyboard: KeyboardHandle<Kind>,
    space: Space<Kind>,
    output: Output,
    pointer_position: (f64, f64),
}

impl PointerTarget<Kind> for CompositorState {
    fn motion(&mut self, _: &Seat<Kind>, event: &MotionEvent<Kind>) {
        self.pointer_position = event.location();
        println!("🖱 Pointer moved to: {:?}", self.pointer_position);
    }

    fn button(&mut self, _: &Seat<Kind>, event: &smithay::input::pointer::ButtonEvent<Kind>) {
        println!("🔘 Mouse button pressed: {:?}", event.button);
    }
}

impl KeyboardTarget<Kind> for CompositorState {
    fn key(&mut self, _: &Seat<Kind>, event: &KeyEvent) {
        println!("⌨️ Key pressed: {:?}", event.key_code);
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

        display.flush_clients()?;
    }
}

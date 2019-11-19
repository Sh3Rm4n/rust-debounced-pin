//! A debounced pin example using the the rtfm framework.
//! Target board: STM32F3DISCOVERY

// Handle the cases where the example is build with the wrong target architecture
#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_main)]
#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {
    eprintln!("Error:");
    eprintln!("\tExample does not work with choosen target_arch.");
    eprintln!("\tBuild with for example --target thumbv7em-none-eabihf instead!");
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
use {
    debounced_pin::prelude::*,
    debounced_pin::ActiveHigh,
    panic_halt as _,
    stm32f3xx_hal::{
        gpio::{gpioa::PA0, gpioe::PE13, Floating, Input, Output, PushPull},
        hal::digital::v2::OutputPin,
        prelude::*,
        stm32::{self, EXTI, TIM1},
        timer::{Event, Timer},
    },
};

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[rtfm::app(device = stm32f3xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        user_button: DebouncedInputPin<PA0<Input<Floating>>, ActiveHigh>,
        led: PE13<Output<PushPull>>,
        external_interrupt: EXTI,
        timer: Timer<TIM1>,
        #[init(false)]
        led_state: bool,
    }
    #[init]
    fn init(_cx: init::Context) -> init::LateResources {
        let dp = stm32::Peripherals::take().unwrap();

        let mut rcc = dp.RCC.constrain();
        let mut flash = dp.FLASH.constrain();
        let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);

        let clocks = rcc.cfgr.freeze(&mut flash.acr);

        let external_interrupt = dp.EXTI;
        // Enable external interrupt for the user_button
        external_interrupt.imr1.write(|w| w.mr0().set_bit());
        external_interrupt.rtsr1.write(|w| w.tr0().set_bit());

        // Intialize the timer periphery
        let mut timer = Timer::tim1(dp.TIM1, 1.mhz(), clocks, &mut rcc.apb2);
        // Enable timer interrupt
        timer.listen(Event::Update);
        // Timer is initially started
        timer.stop();

        // Configure led, which is the "south" led on the stm32 discovery board
        let mut led = gpioe
            .pe13
            .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

        led.set_high().unwrap();

        // Configure the user_button, which get's debounced
        let user_button = gpioa
            .pa0
            .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);

        // button is externally pulled down, and is pulled up via a button press
        let user_button = DebouncedInputPin::new(user_button, ActiveHigh);

        init::LateResources {
            user_button,
            led,
            timer,
            external_interrupt,
        }
    }

    #[task(binds = TIM1_UP_TIM16, spawn = [button_action], resources = [user_button, timer])]
    fn button_poll(cx: button_poll::Context) {
        // Check if pin is debounced
        match cx.resources.user_button.update().unwrap() {
            DebounceState::Active => {
                // stop polling button state
                cx.resources.timer.stop();
                // start button action task
                cx.spawn.button_action().unwrap();
            }
            DebounceState::Reset => {
                // stop polling button state
                cx.resources.timer.stop();
            }
            // Keep debouncing
            DebounceState::Debouncing => (),
            // This task should'nt be executed, when task is not active
            DebounceState::NotActive => (),
        }
        cx.resources.timer.clear_update_interrupt_flag();
    }

    #[task(resources = [led, led_state])]
    fn button_action(cx: button_action::Context) {
        if *cx.resources.led_state {
            cx.resources.led.set_low().unwrap();
        } else {
            cx.resources.led.set_high().unwrap();
        }
        *cx.resources.led_state = !*cx.resources.led_state
    }

    // EXTI0 is the interrupt for a pin interrupt of Px0
    #[task(binds = EXTI0, resources = [timer, external_interrupt])]
    fn button_interrupt(cx: button_interrupt::Context) {
        // Start button poll timer
        cx.resources.timer.start(1.mhz());
        // Clear interrupt pending flag
        cx.resources
            .external_interrupt
            .pr1
            .write(|w| w.pr0().set_bit());
    }

    extern "C" {
        fn UART4_EXTI34();
    }
};

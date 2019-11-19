//! A debounced pin example using isr rountines.
//! Target board: STM32F3DISCOVERY

// #![deny(unused_imports)]
// #![deny(dead_code)]
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
    core::{cell::RefCell, ops::DerefMut},
    cortex_m::asm,
    cortex_m::interrupt::Mutex,
    cortex_m_rt::entry,
    debounced_pin::prelude::*,
    debounced_pin::ActiveHigh,
    panic_semihosting as _,
    stm32f3xx_hal::{
        gpio::{gpioa::PA0, Floating, Input},
        hal::digital::v2::{InputPin, OutputPin},
        interrupt,
        prelude::*,
        stm32::{self, EXTI, TIM1},
        timer::{Event, Timer},
    },
};

#[cfg(all(target_arch = "arm", target_os = "none"))]
static USER_BUTTON: Mutex<RefCell<Option<DebouncedInputPin<PA0<Input<Floating>>, ActiveHigh>>>> =
    Mutex::new(RefCell::new(None));

#[cfg(all(target_arch = "arm", target_os = "none"))]
static TIMER: Mutex<RefCell<Option<Timer<TIM1>>>> = Mutex::new(RefCell::new(None));

#[cfg(all(target_arch = "arm", target_os = "none"))]
static EXT_ITR: Mutex<RefCell<Option<EXTI>>> = Mutex::new(RefCell::new(None));

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[entry]
fn main() -> ! {
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

    cortex_m::interrupt::free(|ref mut cs| {
        EXT_ITR.borrow(cs).replace(Some(external_interrupt));

        TIMER.borrow(cs).borrow_mut().replace(timer);
    });

    let mut led_state = false;

    // Configure led, which is the "south" led on the stm32 discovery board
    let mut led = gpioe
        .pe13
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    led.set_low().unwrap();

    cortex_m::interrupt::free(|cs| {
        // Configure the user_button, which get's debounced
        let user_button = gpioa
            .pa0
            .into_floating_input(&mut gpioa.moder, &mut gpioa.pupdr);

        // button is externally pulled down, and is pulled up via a button press
        let user_button = DebouncedInputPin::new(user_button, ActiveHigh);
        USER_BUTTON.borrow(cs).borrow_mut().replace(user_button);
    });

    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::TIM1_UP_TIM16);
        cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::EXTI0);
    }

    loop {
        cortex_m::interrupt::free(|cs| {
            if let Some(ref user_button) = USER_BUTTON.borrow(cs).borrow().as_ref() {
                if user_button.is_high().unwrap() {
                    // Toggle the led
                    if led_state {
                        led.set_low().unwrap();
                    } else {
                        led.set_high().unwrap();
                    }
                    led_state = !led_state
                }
            }
        });
        // Sleep and only wake up on interrupt
        asm::wfi();
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[interrupt]
fn TIM1_UP_TIM16() {
    cortex_m::interrupt::free(|cs| {
        if let (Some(ref mut timer), Some(ref mut user_button)) = (
            TIMER.borrow(cs).borrow_mut().deref_mut(),
            USER_BUTTON.borrow(cs).borrow_mut().deref_mut(),
        ) {
            if user_button.update().unwrap() == DebounceState::Reset {
                timer.stop();
            }
            timer.clear_update_interrupt_flag();
        }
    });
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[interrupt]
fn EXTI0() {
    cortex_m::interrupt::free(|cs| {
        if let (Some(ref mut timer), Some(ref external_interrupt)) = (
            TIMER.borrow(cs).borrow_mut().deref_mut(),
            EXT_ITR.borrow(cs).borrow().as_ref(),
        ) {
            // Start button poll timer
            timer.start(1.mhz());
            // Clear interrupt pending flag
            external_interrupt.pr1.write(|w| w.pr0().set_bit());
        }
    });
}

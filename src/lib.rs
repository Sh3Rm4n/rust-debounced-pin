//! Adds a wrapper for an `InputPin` that debounces it's `is_high()` and `is_low()` methods.

#![no_std]

use core::marker::PhantomData;
use embedded_hal::digital::v2::InputPin;

/// Unit struct for active-low pins.
pub struct ActiveLow;

/// Unit struct for active-high pins.
pub struct ActiveHigh;

/// The debounce state of the `update()` method
#[derive(PartialEq)]
pub enum DebounceState {
    /// The pin state is active, but not debounced
    Debouncing,
    /// The pin state is not active, the counter is reset
    Reset,
    /// The pin state is high and is debounced
    Active,
}

/// A debounced input pin.
///
/// Implements approach 1 from [here](http://www.labbookpages.co.uk/electronics/debounce.html#soft)
/// ([archived 2018-09-03](https://web.archive.org/web/20180903142143/http://www.labbookpages.co.uk/electronics/debounce.html#soft)).
///
/// Requires `update()` to be called every ~1ms.
pub struct DebouncedInputPin<T: InputPin, A> {
    /// The wrapped pin.
    pub pin: T,

    /// Whether the pin is active-high or active-low.
    activeness: PhantomData<A>,

    /// The counter.
    counter: i8,

    /// The debounced pin state.
    state: bool,
}

impl<T: InputPin, A> DebouncedInputPin<T, A> {
    /// Initializes a new debounced input pin.
    pub fn new(pin: T, _activeness: A) -> Self {
        Self {
            pin,
            activeness: PhantomData,
            counter: 0,
            state: false,
        }
    }
}

impl<T: InputPin, A> InputPin for DebouncedInputPin<T, A> {
    type Error = T::Error;

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.state)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.state)
    }
}

impl<T: InputPin> DebouncedInputPin<T, ActiveHigh> {
    /// Initializes a new `ActiveHigh` debounced input pin.
    pub fn active_high(pin: T) -> Self {
        Self {
            pin,
            activeness: PhantomData,
            counter: 0,
            state: false,
        }
    }

    /// Updates the debounce logic.
    ///
    /// Needs to be called every ~1ms.
    pub fn update(&mut self) -> Result<DebounceState, <Self as InputPin>::Error> {
        if self.pin.is_low()? {
            self.counter = 0;
            self.state = false;
            Ok(DebounceState::Reset)
        } else if self.counter < 10 {
            self.counter += 1;
            Ok(DebounceState::Debouncing)
        } else {
            // Max count is reached
            self.state = true;
            Ok(DebounceState::Active)
        }
    }
}

impl<T: InputPin> DebouncedInputPin<T, ActiveLow> {
    /// Initializes a new `ActiveLow` debounced input pin.
    pub fn active_low(pin: T) -> Self {
        Self {
            pin,
            activeness: PhantomData,
            counter: 0,
            state: true,
        }
    }
    /// Updates the debounce logic.
    ///
    /// Needs to be called every ~1ms.
    pub fn update(&mut self) -> Result<DebounceState, <Self as InputPin>::Error> {
        if self.pin.is_high()? {
            self.counter = 0;
            self.state = true;
            Ok(DebounceState::Reset)
        } else if self.counter < 10 {
            self.counter += 1;
            Ok(DebounceState::Debouncing)
        } else {
            // Max count is reached
            self.state = false;
            Ok(DebounceState::Active)
        }
    }
}

#[cfg(test)]
mod tests;

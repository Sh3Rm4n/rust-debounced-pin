//! Adds a wrapper for an `InputPin` that debounces it's `is_high()` and `is_low()` methods.

#![no_std]

use core::marker::PhantomData;
use embedded_hal::digital::v2::InputPin;

/// Unit struct for active-low pins.
pub struct ActiveLow;

/// Unit struct for active-high pins.
pub struct ActiveHigh;

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

    /// Maximum number of times, the pin has to be polled and be continuously active to
    /// change it's debounce state.
    max_counts: i8,

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
            max_counts: 10,
            state: false,
        }
    }

    /// Change the number of times, the pin has to be polled and be continuously active to
    /// change it's debounce state.
    pub fn set_poll_amounts(&mut self, max_counts: i8) {
        self.max_counts = max_counts;
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
            max_counts: 10,
            state: false,
        }
    }

    /// Updates the debounce logic.
    ///
    /// Needs to be called every ~1ms.
    pub fn update(&mut self) -> Result<(), <Self as InputPin>::Error> {
        if self.pin.is_low()? {
            self.counter = 0;
            self.state = false;
        } else if self.counter < self.max_counts {
            self.counter += 1;
        }

        if self.counter == self.max_counts {
            self.state = true;
        }

        Ok(())
    }
}

impl<T: InputPin> DebouncedInputPin<T, ActiveLow> {
    /// Initializes a new `ActiveLow` debounced input pin.
    pub fn active_low(pin: T) -> Self {
        Self {
            pin,
            activeness: PhantomData,
            counter: 0,
            max_counts: 10,
            state: true,
        }
    }
    /// Updates the debounce logic.
    ///
    /// Needs to be called every ~1ms.
    pub fn update(&mut self) -> Result<(), <Self as InputPin>::Error> {
        if self.pin.is_high()? {
            self.counter = 0;
            self.state = true;
        } else if self.counter < self.max_counts {
            self.counter += 1;
        }

        if self.counter == self.max_counts {
            self.state = false;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;

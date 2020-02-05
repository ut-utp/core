//! Stack allocated FIFO. (TODO)

use core::mem::{transmute, MaybeUninit, size_of};
use core::convert::TryInto;

use bytes::{Buf, BufMut};

// Note: Capacity is constant ~an associated type~ so that the transition to
// const generics (once that lands on stable) will be mostly seamless.

pub(in super) mod FifoConfig {
    pub const CAPACITY: usize = 256;
    pub type Cur = u16;
}

use FifoConfig::{CAPACITY, Cur};

pub struct Fifo<T> {
    data: [T; CAPACITY], // Pick this so that we can read in the largest message. Also have compile time asserts that check that all the messages fit within a buffer of this size! (TODO) (shouldn't be too bad since we have one message...)
    length: usize,
    starting: Cur,
    ending: Cur,   // Points to the next empty slot.
}

// TODO: Both of these aren't supported yet. When they are supported const
// generics will also likely be supported. Meaning these are entirely pointless.
// impl<T: Sized> Fifo<T> {
//     const CAPACITY: usize = 256; // TODO: compile time assert that this is in [1, Cur::MAX]. // (REMOVE)
//     type Cur = u16;
// }

// If this doesn't hold, the as in the next check isn't guaranteed not to lose
// bits.
sa::const_assert!(size_of::<Cur>() <= size_of::<usize>());

// `FifoConfig::CAPACITY` ∈ [1, Cur::MAX]
sa::const_assert!(CAPACITY <= Cur::max_value() as usize);
sa::const_assert!(CAPACITY >= 1);

impl<T: Default> Default for Fifo<T> {
    fn default() -> Self {
        Self::new_with_default()
    }
}

impl<T: Default> Fifo<T> {
    pub const fn new_with_default() -> Self {
        Self::new([T::default(); CAPACITY])
    }
}

impl<T: Copy> Fifo<T> {
    pub const fn new_with_value(val: T) -> Self {
        Self::new([val; CAPACITY])
    }
}

impl<T> Fifo<T> {
    pub const fn capacity() -> usize {
        CAPACITY
    }

    pub const fn new(data: [T; CAPACITY]) -> Self {
        Self {
            data,
            length: 0,
            starting: 0,
            ending: 0
        }
    }

    pub const fn is_empty(&self) -> bool { self.length == 0 }
    pub const fn is_full(&self) -> bool { self.length == CAPACITY }

    pub const fn length(&self) -> usize { self.length }
    pub const fn remaining(&self) -> usize { CAPACITY - self.length }

    // fn increment(pos: Cur) -> Cur {
    //     if pos == ((CAPACITY - 1) as Cur) {
    //         0
    //     } else {
    //         pos + 1
    //     }
    // }

    const fn add(pos: Cur, num: Cur) -> Cur {
        (((pos as usize) + (num as usize)) % CAPACITY) as Cur
    }

    pub fn push(&mut self, datum: T) -> Result<(), ()> {
        if self.is_full() {
            Err(())
        } else {
            self.length += 1;
            self.data[self.ending as usize] = datum;
            self.ending = Self::add(self.ending, 1);

            Ok(())
        }
    }

    pub fn peek(&self) -> Result<T, ()> {
        if self.is_empty() {
            Err(())
        } else {
            Ok(self.data[self.starting as usize])
        }
    }

    pub fn pop(&mut self) -> Result<T, ()> {
        let datum = self.peek()?;

        self.advance(1);
        Ok(datum)
    }

    pub fn bytes(&self) -> &[T] {
        // starting == ending can either mean a full fifo or an empty one
        if self.is_empty() {
            &[]
        } else {
            if self.ending > self.starting {
                &self.data[(self.starting as usize)..(self.ending as usize)]
            } else if self.ending <= self.starting {
                // Gotta do it in two parts then.
                &self.data[(self.starting as usize)..]
            } else { unreachable!() }
        }
    }

    // fn advance(&mut self, num: Cur) -> Result<(), ()> {
    fn advance(&mut self, num: Cur) {
        assert!((num as usize) <= self.length);

        self.length -= (num as usize);
        self.starting = Self::add(self.starting, num);
    }
}

impl Buf for Fifo<u8> {
    fn remaining(&self) -> usize {
        self.length()
    }

    fn bytes(&self) -> &[u8] {
        self.bytes()
    }

    fn advance(&mut self, count: usize) {
        self.advance(count.try_into().unwrap());
    }
}

impl BufMut for Fifo<u8> {
    fn remaining_mut(&self) -> usize {
        self.remaining()
    }

    unsafe fn advance_mut(&mut self, cnt: usize) {
        if cnt > self.remaining_mut() {
            panic!("Attempted to write more than the buffer can accommodate.");
        }

        // If cnt is less than the number of slots we've got and the number of
        // slots we've got is representable by the cursor size, this should be
        // fine.
        let cnt_cur: Cur = cnt.try_into().unwrap();

        // Should also be fine (for overflow) if the check above doesn't panic.
        // We also won't exceed the capacity of the fifo if we're not writing
        // more than number of slots that are remaining (the above check).
        self.length += cnt;
        self.ending = Self::add(self.ending, cnt_cur);
    }

    fn bytes_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        let slice = if self.is_empty() {
            &mut self.data
        } else {
            if self.ending <= self.starting {
                &mut self.data[(self.ending as usize)..(self.starting as usize)]
            } else if self.ending > self.starting {
                // Gotta do it in two parts then.
                &mut self.data[(self.ending as usize)..]
            } else { unreachable!() }
        };

        // This is probably safe since `MaybeUninit<T>` and `T` are guaranteed
        // to have the same representation (size, alignment, and ABI).
        //
        // Probably because as per the `MaybeUninit` union docs, types that
        // contain a MaybeUninit don't necessarily have to have the same
        // representation as types that just contain `T`. There's an assert for
        // this at the bottom of this file.
        unsafe { transmute::<&mut [u8], &mut [MaybeUninit<u8>]>(slice) }
    }
}

// Note: if we switch to const generics for `CAPACITY`, move this to the
// constructor.
sa::assert_eq_size!(&mut [MaybeUninit<u8>], &mut [u8]);
sa::assert_eq_size!([MaybeUninit<u8>; CAPACITY], [u8; CAPACITY]);
sa::assert_eq_align!([MaybeUninit<u8>; CAPACITY], [u8; CAPACITY]);

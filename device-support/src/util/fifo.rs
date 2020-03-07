//! Stack allocated FIFO. (TODO)

use core::{
    iter::ExactSizeIterator,
    mem::{replace, size_of, transmute_copy, MaybeUninit},
};

// Note: Capacity is a constant so that the transition to const generics (once
// that lands on stable) will be not terrible painful.

pub(super) mod fifo_config {
    use core::mem::size_of;

    pub const CAPACITY: usize = 256;
    pub type Cur = u16;

    // If this doesn't hold, the as in the next check isn't guaranteed not to
    // lose bits.
    sa::const_assert!(size_of::<Cur>() <= size_of::<usize>());

    // `FifoConfig::CAPACITY` ∈ [1, Cur::MAX]
    sa::const_assert!(CAPACITY <= Cur::max_value() as usize);
    sa::const_assert!(CAPACITY >= 1);
}

pub use fifo_config::{Cur, CAPACITY};

pub struct Fifo<T> {
    data: [MaybeUninit<T>; CAPACITY],
    length: usize,
    /// Points to the next slot that holds data.
    /// Valid when `length` > 0.
    starting: Cur,
    /// Points to the next empty slot.
    /// Valid when `length` < CAPACITY.
    ending: Cur,
}

impl<T> Default for Fifo<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Fifo<T> {
    /// Creates an empty `Fifo`.
    pub fn new() -> Self {
        // This is really a job for `MaybeUninit::uninit_array` but alas, it is
        // not yet stable (it needs const generics).
        let data = MaybeUninit::<[MaybeUninit<T>; CAPACITY]>::uninit();

        // This is safe because we can assume that _arrays_ have the same memory
        // representation as a literal composite of their elements (so an array
        // of MaybeUninits contains only the bits belonging to the MaybeUninit
        // elements: there aren't other bits we need to worry about) and because
        // we can then safely call `assume_init` on any `MaybeUninit<_>` type
        // since `MaybeUninit<_>` (which is aware that the type inside it may
        // not be valid for the bit representation it current has) is valid for
        // all bit representations.
        #[allow(unsafe_code)]
        let data = unsafe { data.assume_init() };

        Self {
            data,
            length: 0,
            starting: 0,
            ending: 0,
        }
    }

    /// Creates a new `Fifo` and is const.
    ///
    /// This can only be const because `CAPACITY` is a constant (and not a
    /// const generic parameter). Hopefully by the time const generics actually
    /// land we'll have repeating const expressions or a const `assume_init`
    /// function.
    pub const fn new_const() -> Self {
        // This isn't great. Past attempts are in the git history for posterity.
        // I'm not convinced it's possible to do better without using
        // `proc-macro-hack`.

        macro_rules! repeat {
            (($t:expr) => { $($rest:ident)* }) => {
                [$(
                    { #[allow(unused)] let $rest = 0; $t },
                )*]
            };
        }

        let data = repeat!((MaybeUninit::uninit()) => {
            t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t
            t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t
            t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t
            t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t
            t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t
            t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t
            t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t
            t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t t
        });

        Self {
            data,
            length: 0,
            starting: 0,
            ending: 0,
        }
    }

    /// The maximum number of elements the `Fifo` can hold.
    pub const fn capacity() -> usize {
        CAPACITY
    }

    /// Whether the `Fifo` is empty or not.
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Whether the `Fifo` is full or not.
    pub const fn is_full(&self) -> bool {
        self.length == CAPACITY
    }

    /// Number of elements currently in the `Fifo`.
    pub const fn length(&self) -> usize {
        self.length
    }

    /// Number of open slots the `Fifo` currently has.
    pub const fn remaining(&self) -> usize {
        CAPACITY - self.length
    }

    // A wheel function.
    // Note: this is not overflow protected!
    // TODO: spin off the protected wheel into its own crate and use that
    // here!
    const fn add(pos: Cur, num: Cur) -> Cur {
        // Note: usize is guaranteed to be ≥ to Cur in size so the cast is
        // guaranteed not to lose bits.
        (((pos as usize) + (num as usize)) % CAPACITY) as Cur
    }

    /// Adds a value to the `Fifo`, if possible.
    ///
    /// Returns `Err(())` if the `Fifo` is currently full.
    pub fn push(&mut self, datum: T) -> Result<(), ()> {
        if self.is_full() {
            Err(())
        } else {
            self.length += 1;
            self.data[self.ending as usize] = MaybeUninit::new(datum);
            self.ending = Self::add(self.ending, 1);

            Ok(())
        }
    }

    /// Gives a reference to the next value in the `Fifo`, if available.
    ///
    /// This function doesn't remove the value from the `Fifo`; use `pop` to do
    /// that.
    ///
    /// [`pop`]: Fifo::pop
    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let datum: *const T = self.data[self.starting as usize].as_ptr();

            // Leaning on our invariants here; if we haven't just returned this
            // specific value was inserted (in a valid state) so we can safely
            // assume that this value is initialized.
            #[allow(unsafe_code)]
            Some(unsafe { &*datum })
        }
    }

    // Updates the starting and length count to 'consume' some number of
    // elements.
    fn advance(&mut self, num: Cur) {
        assert!((num as usize) <= self.length);

        self.length -= num as usize;
        self.starting = Self::add(self.starting, num);
    }

    /// Pops a value from the Fifo, if available.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let datum = replace(
                &mut self.data[self.starting as usize],
                MaybeUninit::uninit(),
            );

            self.advance(1);

            // As with peek, we trust our invariants to keep us safe here.
            // Because this value must have been initialized for us to get here,
            // we know this is safe.
            #[allow(unsafe_code)]
            Some(unsafe { datum.assume_init() })
        }
    }

    /// Returns a slice consisting of the data currently in the `Fifo` without
    /// removing it.
    pub fn as_slice(&self) -> &[T] {
        // starting == ending can either mean a full fifo or an empty one so
        // we use our length field to handle this case separately
        if self.is_empty() {
            &[]
        } else {
            if self.ending > self.starting {
                let s = &self.data
                    [(self.starting as usize)..(self.ending as usize)];

                // Again, leaning on our invariants and assuming this is all
                // init-ed data.
                // TODO: not confident the transmute is actually safe here.
                //
                // [MaybeUninit<T>] and [T]
                // and
                // &[MaybeUninit<T>] and &[T]
                // have the same representations right?
                //
                // This is probably safe since `MaybeUninit<T>` and `T` are
                // guaranteed to have the same representation (size, alignment,
                // and ABI).
                // Probably because as per the `MaybeUninit` union docs, types
                // that contain a MaybeUninit don't necessarily have to have the
                // same representation as types that just contain `T`. There's
                // an assert for this at the bottom of this file.
                #[allow(unsafe_code)]
                unsafe {
                    transmute(s)
                }
            } else if self.ending <= self.starting {
                // Gotta do it in two parts then.
                let s = &self.data[(self.starting as usize)..];

                // Same as above.
                #[allow(unsafe_code)]
                unsafe {
                    transmute(s)
                }
            } else {
                unreachable!()
            }
        }
    }
}

impl<T: Clone> Fifo<T> {
    /// Because we cannot take ownership of the slice, this is only available
    /// for `Clone` (and, thus, `Copy`) types.
    ///
    /// This operation is 'atomic': either all of the slice gets pushed (if
    /// there is space for it) or none of it does.
    ///
    /// If the slice cannot be pushed in its entirety, this function returns
    /// `Err(())`.
    pub fn push_slice(&mut self, slice: &[T]) -> Result<(), ()> {
        if self.remaining() < slice.len() {
            Err(())
        } else {
            for v in slice.iter().cloned() {
                self.push(v).expect("fifo: internal error")
            }

            Ok(())
        }
    }
}

impl<T> Fifo<T> {
    /// Like [`push_slice`] this function is 'atomic': it will either succeed
    /// (and in this case push the iterator in its entirety) or it will leave
    /// the `Fifo` unmodified.
    ///
    /// Because we want this property we need to know the length of the iterator
    /// beforehand and that's where [`ExactSizeIterator`] comes in. With a
    /// normal [`Iterator`] we can't know the length of the iterator until we've
    /// consumed it, but `ExactSizeIterator`s just tell us.
    ///
    /// This particular function will require an iterator that transfers
    /// ownership of the values (i.e the kind you get when you call [`drain`] on
    /// a [`Vec`]). If this is not what you want (and if your type is
    /// [`Clone`]able), try [`push_iter_ref`].
    ///
    /// Like [`push_slice`], this will return `Err(())` if it is unable to push
    /// the entire iterator. Note that we take a mutable reference to your
    /// iterator, so in the event that we are not able to push your values, they
    /// are not just dropped (you can try again or do something else with your
    /// values).
    ///
    /// [`push_slice`]: Fifo::push_slice
    /// [`push_iter_ref`]: Fifo::push_iter_ref
    /// [`ExactSizeIterator`]: core::iter::ExactSizeIterator
    /// [`Iterator`]: core::iter::Iterator
    /// [`Clone`]: core::clone::Clone
    /// [`Vec`]: alloc::vec::Vec
    /// [`drain`]: alloc::vec::Vec::drain
    pub fn push_iter<I: ExactSizeIterator<Item = T>>(
        &mut self,
        iter: &mut I,
    ) -> Result<(), ()> {
        let len = iter.len();

        if self.remaining() < len {
            Err(())
        } else {
            for _ in 0..len {
                self.push(
                    iter.next().expect("ExactSizeIterator length was wrong!"),
                )
                .expect("fifo: internal error")
            }

            Ok(())
        }
    }
}

impl<'a, T: Clone + 'a> Fifo<T> {
    /// The version of [`push_iter`] that doesn't need ownership of the `T`
    /// values your iterator is yielding.
    ///
    /// This works like [`push_slice`] does and thus this also only works for
    /// types that implement [`Clone`].
    ///
    /// Returns `Err(())` if unable to push the entire iterator.
    ///
    /// [`push_iter`]: Fifo::push_iter
    /// [`push_slice`]: Fifo::push_slice
    /// [`Clone`]: core::clone::Clone
    pub fn push_iter_ref<'i: 'a, I: ExactSizeIterator<Item = &'a T>>(
        &mut self,
        iter: &'i mut I,
    ) -> Result<(), ()> {
        self.push_iter(&mut iter.cloned())
    }
}

impl<T: Clone> Fifo<T> {
    /// Useful for generating arrays out of a `Clone`able (but not `Copy`able)
    /// value to pass into `Fifo::put_slice`.
    pub fn array_init_using_clone(val: T) -> [T; CAPACITY] {
        // MaybeUninit is always properly initialized.
        // Note: this is _the_ use case for `MaybeUninit::uninit_array` which is
        // not yet stable (blocked on const-generics like all the shiny things).
        #[allow(unsafe_code)]
        let mut inner: [MaybeUninit<T>; CAPACITY] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for elem in &mut inner[..] {
            *elem = MaybeUninit::new(val.clone());
        }

        assert_eq!(
            size_of::<[MaybeUninit<T>; CAPACITY]>(),
            size_of::<[T; CAPACITY]>()
        );

        // Because we've initialized every element manually, this is safe.
        // Additionally, the assert above (which will always be true in our
        // case) is a way for us to be extremely certain that `transmute_copy`'s
        // invariant is upheld.
        #[allow(unsafe_code)]
        unsafe {
            transmute_copy(&inner)
        }
    }
}

using_alloc! {
    use core::mem::transmute;
    use core::convert::TryInto;

    use bytes::{Buf, BufMut};

    impl Buf for Fifo<u8> {
        fn remaining(&self) -> usize {
            self.length()
        }

        fn bytes(&self) -> &[u8] {
            self.as_slice()
        }

        fn advance(&mut self, count: usize) {
            self.advance(count.try_into().unwrap());
        }
    }

    impl BufMut for Fifo<u8> {
        fn remaining_mut(&self) -> usize {
            self.remaining()
        }

        #[allow(unsafe_code)] // Nothing _we_ do here is unsafe..
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
            if self.is_empty() {
                &mut self.data
            } else {
                if self.ending <= self.starting {
                    &mut self.data[(self.ending as usize)..(self.starting as usize)]
                } else if self.ending > self.starting {
                    // Gotta do it in two parts then.
                    &mut self.data[(self.ending as usize)..]
                } else { unreachable!() }
            }
        }
    }
}

// Note: if we switch to const generics for `CAPACITY`, move this to the
// constructor.
sa::assert_eq_size!(&mut [MaybeUninit<u8>], &mut [u8]);
sa::assert_eq_size!([MaybeUninit<u8>; CAPACITY], [u8; CAPACITY]);
sa::assert_eq_align!([MaybeUninit<u8>; CAPACITY], [u8; CAPACITY]);

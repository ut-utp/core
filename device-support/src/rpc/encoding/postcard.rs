//! TODO!

use crate::util::Fifo;

use lc3_traits::control::rpc::{Encode, Decode, RequestMessage, ResponseMessage};

use serde::{Serialize, Deserialize};
use postcard::flavors::{SerFlavor, Cobs, Slice};
use postcard::serialize_with_flavor;
use postcard::take_from_bytes_cobs;

use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::IndexMut;
use core::convert::{AsRef, AsMut};

// TODO: have be able to take inputs?
// no: they're closures; just capture.

mod encode {
    use super::*;

    #[derive(Debug, Default)]
    pub struct PostcardEncode<Inp, F, Func>
    where
        Inp: ?Sized + Debug + Serialize,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
        Func: FnMut() -> F,
    {
        // flavor: F,
        flavor_func: Func,
        _i: PhantomData<Inp>,
    }

    impl<Inp, F, Func> PostcardEncode<Inp, F, Func>
    where
        Inp: ?Sized + Debug + Serialize,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
        Func: FnMut() -> F,
    {
        // Once we can have const fns with real trait bounds this can be const.
        // pub /*const*/ fn new(flavor: F) -> Self {
        //     Self {
        //         flavor,
        //         _i: PhantomData,
        //     }
        // }

        pub fn new(flavor_func: Func) -> Self {
            Self {
                flavor_func,
                _i: PhantomData
            }
        }
    }

    impl<Inp, I, CFunc> PostcardEncode<Inp, Cobs<I>, CFunc>
    where
        Inp: ?Sized + Debug + Serialize,
        I: SerFlavor,
        I: IndexMut<usize, Output = u8>,
        <I as SerFlavor>::Output: Debug,
        CFunc: FnMut() -> Cobs<I>,
    {
        pub /*const*/ fn with_cobs(mut inner_flavor_func: impl FnMut() -> I) -> PostcardEncode<Inp, Cobs<I>, impl FnMut() -> Cobs<I>> {
            // Ok(PostcardEncode::new(Cobs::try_new(inner_flavor)?))

            PostcardEncode::new(move || Cobs::try_new((inner_flavor_func)()).unwrap())
        }
    }

    // impl<'a, Inp> PostcardEncode<Inp, Cobs<Slice<'a>>>
    // where
    //     Inp: ?Sized + Debug + Serialize,
    // {
    //     pub /*const*/ fn with_slice(buffer: &'a mut [u8]) -> postcard::Result<Self> {
    //         Ok(PostcardEncode::new(Cobs::try_new(Slice::new(buffer))?))
    //     }
    // }

    // pub type PostcardFifoCobs<Inp> = PostcardEncode<Inp, Cobs<Fifo<u8>>>;

    impl<Inp, Func> PostcardEncode<Inp, Cobs<Fifo<u8>>, Func>
    where
        Inp: ?Sized + Debug + Serialize,
        Func: FnMut() -> Cobs<Fifo<u8>>,
    {
        // pub /*const*/ fn with_fifo() -> postcard::Result<Self> {
            // Ok(PostcardEncode::new(Cobs::try_new(Fifo::new())?))
        // pub /*const*/ fn with_fifo() -> postcard::Result<Self> {
        pub /*const*/ fn with_fifo() -> PostcardEncode<Inp, Cobs<Fifo<u8>>, impl FnMut() -> Cobs<Fifo<u8>>> {

            PostcardEncode::<Inp, _, Func>::with_cobs(|| Fifo::new())
        }
    }

    impl<Inp, F, Func> Encode<&Inp> for PostcardEncode<Inp, F, Func>
    where
        Inp: ?Sized + Debug + Serialize,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
        Func: FnMut() -> F,
    {
        type Encoded = <F as SerFlavor>::Output;

        fn encode(&mut self, message: &Inp) -> <F as SerFlavor>::Output {
            // postcard::
            serialize_with_flavor(message, (self.flavor_func)())
                .expect("a successful encode")
        }
    }

    impl<Inp, F, Func> Encode<Inp> for PostcardEncode<Inp, F, Func>
    where
        Inp: Debug + Serialize,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
        Func: FnMut() -> F,
    {
        type Encoded = <F as SerFlavor>::Output;

        fn encode(&mut self, message: Inp) -> <F as SerFlavor>::Output {
            serialize_with_flavor(&message, (self.flavor_func)())
                .expect("a successful encode")
        }
    }
}

mod decode {
    use super::*;

    // TODO: have a default like this for PostcardEncode (Cobs<Fifo<u8>>)
    #[derive(Debug, Default)]
    pub struct PostcardDecode<Out, F = Cobs<Fifo<u8>>>
    where
        Out: Debug,
        for<'de> Out: Deserialize<'de>,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
    {
        _f: PhantomData<F>,
        _o: PhantomData<Out>,
    }

    impl<Out, F> PostcardDecode<Out, F>
    where
        Out: Debug,
        for<'de> Out: Deserialize<'de>,
        F: SerFlavor,
        <F as SerFlavor>::Output: Debug,
    {
        pub /*const*/ fn new() -> Self {
            Self {
                _f: PhantomData,
                _o: PhantomData,
            }
        }
    }

    // We can't provide full generality because there's no DeFlavor trait.
    // Unclear whether we can to better than the below (Cobs + AsMut). TODO.

    // TODO: this cloning stuff is bad; can we change Decode to give a mutable
    // reference to the encoded data (and switch back to AsMut).

    impl<F, Out> Decode<Out> for PostcardDecode<Out, Cobs<F>>
    where
        Out: Debug,
        for<'de> Out: Deserialize<'de>,
        F: SerFlavor,
        F: IndexMut<usize, Output = u8>,
        Cobs<F>: SerFlavor,
        <Cobs<F> as SerFlavor>::Output: Debug,
        <Cobs<F> as SerFlavor>::Output: AsRef<[u8]>
    {
        type Encoded = <Cobs<F> as SerFlavor>::Output;
        type Err = postcard::Error;

        fn decode(&mut self, encoded: &Self::Encoded) -> Result<Out, Self::Err> {
            // This is bad and is a hack!
            let mut fifo: Fifo<u8> = Fifo::new();
            fifo.push_slice(encoded.as_ref());
            // fifo.push_iter(&mut encoded.as_ref().iter()).unwrap();

            // // TODO: remove this hack!
            // match take_from_bytes_cobs(fifo.as_mut()) {
            //     Ok((ref m, _)) => Ok(Out::clone(m)),
            //     Err(e) => Err(e),
            // }
            // if let Some((m, _)) =  {
            //     Ok(m.clone())
            // }

            take_from_bytes_cobs(fifo.as_mut())
                .map(|(m, _)| m)
        }
    }
}

impl SerFlavor for Fifo<u8> {
    type Output = Self;

    fn try_push(&mut self, data: u8) -> Result<(), ()> {
        self.push(data)
    }

    fn release(self) -> Result<Self::Output, ()> {
        Ok(self)
    }

    fn try_extend(&mut self, data: &[u8]) -> Result<(), ()> {
        self.push_slice(data)
    }
}

pub use encode::*;
pub use decode::*;

//! TODO!

use crate::util::Fifo;

use lc3_traits::control::rpc::{Encode, Decode, RequestMessage, ResponseMessage};

use serde::Serialize;
use postcard::flavors::{SerFlavor, Cobs, Slice};
use postcard::serialize_with_flavor;

use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::IndexMut;

// TODO: have be able to take inputs?
// no: they're closures; just capture.

#[derive(Debug, Default)]
pub struct PostcardEncode<Inp, F, Func>
where
    Inp: ?Sized + Debug + Serialize,
    F: SerFlavor,
    <F as SerFlavor>::Output: Debug,
    Func: Fn() -> F,
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
    Func: Fn() -> F,
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
    CFunc: Fn() -> Cobs<I>,
{
    pub /*const*/ fn with_cobs(inner_flavor_func: impl Fn() -> I) -> PostcardEncode<Inp, Cobs<I>, impl Fn() -> Cobs<I>> {
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
    Func: Fn() -> Cobs<Fifo<u8>>,
{
    // pub /*const*/ fn with_fifo() -> postcard::Result<Self> {
        // Ok(PostcardEncode::new(Cobs::try_new(Fifo::new())?))
    // pub /*const*/ fn with_fifo() -> postcard::Result<Self> {
    pub /*const*/ fn with_fifo() -> PostcardEncode<Inp, Cobs<Fifo<u8>>, impl Fn() -> Cobs<Fifo<u8>>> {

        PostcardEncode::<Inp, _, Func>::with_cobs(|| Fifo::new())
    }
}

impl<Inp, F, Func> Encode<&Inp> for PostcardEncode<Inp, F, Func>
where
    Inp: ?Sized + Debug + Serialize,
    F: SerFlavor,
    <F as SerFlavor>::Output: Debug,
    Func: Fn() -> F,
{
    type Encoded = <F as SerFlavor>::Output;

    fn encode(&mut self, message: &Inp) -> <F as SerFlavor>::Output {
        // postcard::
        serialize_with_flavor(message, (self.flavor_func)())
            .expect("a successful encode")
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

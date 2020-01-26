//! Parts for control's *encoding* layer. Mainly the [`Encode`] and [`Decode`]
//! traits (and some friends).
//!
//! TODO!

//! Note: We used associated types for `Encoded` for these next two traits so
//! that there's a one to one mapping from a (message type, [unit type][^1])
//! pair to an encoded type.
//!
//! We realize that it is desirable to have many encodings for a message type
//! (i.e. `ResponseMessage` could be encoded as a String (JSON) or a byte slice
//! COBS, etc.). This is still supported with these traits; you'll just need to
//! make a different unit type (i.e. `Cobs`) to implement the other
//! `ResponseMessage` encodings. We wanted users of this machinery to be able to
//! pick the encoding/decoding by specifying a concrete type (i.e. Enc =
//! `JsonEnc`) and we felt that the 1 (message type, unit type) -> (enc type)
//! restriction was a good way to minimize confusion.
//!
//! Something of the shape `trait Enc<Message, Encoded>` would lift the above
//! restriction.
//!
//! There are other ways to do this that we also considered. One is to have the
//! `Self` type be either the `Encoded` type or the `Message` type. Each of
//! these limits what users of this API can actually do:
//!
//! For `Encoded = Self`:
//! ```rust
//! trait Enc<Message: Debug> { fn encode(message: Message) -> Self }
//! ```
//! Observant readers may notice that the above is basically just `From`.
//!
//! We do away with the unit type and now have a many to one relationship
//! between an encoded type and message types which means users will likely have
//! to specify some types manually (instead of passing in a unit struct), which
//! is not great but is okay (users will have to say, for example,
//! `<JsonString as Enc<ResponseMessage>>`).
//!
//! But more importantly, because of the orphan impl rules, users of this crate
//! won't be able to define encodings/decodings on types they did not implement
//! themselves (i.e. for `JsonString` defined in some other crate (C), I, a user
//! writing crate B that uses this crate (A) cannot write this impl:
//! `impl Enc<ResponseMessage> for JsonString` since I don't own the trait or
//! the concrete type — or any of the types in that line, actually). This is not
//! ideal.
//!
//! For `Message = Self`:
//! ```rust
//! trait Enc<Encoded: Debug> { fn encode(message: Self) -> Encoded }
//! ```
//! This time ^ is basically just `Into`.
//!
//! Again, we do away with the unit type. Now we've got a many to one
//! relationship between a message type and encoded types, which is functionally
//! equivalent to the above, I think.
//!
//! But, again, the orphan impl rules get in our way. Users in downstream crates
//! can't, for example, write impls of `Env` on the message types that are
//! defined in this crate (`ResponseMessage`, `RequestMessage) which
//! significantly limits how useful all of this is.
//!
//! So, ultimately, the below is what we settled on. Marker types (that
//! implementors presumably own) let us get around the orphan impl rules.
//!
//! IIRC, this ends up looking kind of sort of superficially similar to
//! `Serializer`/`Deserializer` from `serde` except `serde` is extremely clever
//! and introduces another layer (the `Serialize` and `Deserialize` traits which
//! are for unserialized types — a.k.a what we call `Message` in the above).
//! This is useful for a couple reasons; both of which you can see below:
//!
//! ```rust,no_run
//! // In this crate (A)
//! pub trait Enc<M: Debug> { type Encd: Debug; fn enc(m: M) -> Self::Encd }
//! pub enum ControlResponse { ... }
//!
//! // In some other crate (B), a user of A
//! struct JsonEnc;
//! impl Enc<A::ControlMessage> for JsonEnc {
//!     type Encd = SomeLibCrate::Json;
//!     ...
//! }
//!
//! // In yet another crate (C) that uses A and B
//! // Say, I want to experiment with another message format...
//! struct TinyControlResponse { ... }
//! // Json is fine as a wire format though, so I'll just use that:
//! // Note: we can actually do this and coherence doesn't get in our way.
//! impl Enc<TinyControlResponse> for JsonEnc {
//!     type Encd = SomeLibCrate::Json;
//!     ...
//! }
//!
//! // But if we tried to do:
//! impl Enc<YetAnotherCrate::SuperCoolResponseFormat> for JsonEnc {
//!     type Encd = SomeLibCrate::Json;
//!     ...
//! }
//! // Oh no! Orphan impl rules!
//!
//! // A bad workaround you can do to get around this is:
//! struct MyJsonEnc;
//! impl Enc<YetAnotherCrate::SuperCoolResponseFormat> for MyJsonEnc {
//!     type Encd = SomeLibCrate::Json;
//!     ...
//! }
//! // This is not great since it means you can no longer just say "use
//! //`JsonEnc`"; instead you've got to find the right unit type for that
//! // particular message with sucks.
//! ```
//! So, some things to notice in the above:
//!   - You totally can make your own message format. For now we're going to
//!     require that it can be mapped to `ResponseMessage` and `RequestMessage`
//!     because they have defined mapping to and from Control function calls and
//!     because we don't have a trait to enforce that arbitrary types provide
//!     this — this perhaps makes this much less interesting, but it's a start.
//!   - We aren't totally immune from orphan impl rules; you still do have to
//!     own either the encoding unit type or the message type. `serde` doesn't
//!     actually get around this either but it has a (also very clever)
//!     workaround where, if you've got a type in another crate that doesn't
//!     impl Serialize/Deserialize, you can make a copy of that type in your
//!     crate that does impl it *but* have the Serialize and Deserialize impls
//!     actually produce the type in the external crate; compile time checks to
//!     ensure that the type was copied correctly + support for getters/setters
//!     (i.e. private fields). I'm slightly befuddled by how `serde` gets around
//!     its `Deserialize` trait returning `Self`... maybe it doesn't except for
//!     things that include remote types within them (i.e. you still need a new
//!     type). More info [here](https://github.com/serde-rs/serde/pull/858).
//!   - If you create a serialization format, you need to implement it for every
//!     message format that you wish to serialize/deserialize. In other words,
//!     you need S * M impls. `serde`, because it decouples Serializers from
//!     types that can be serialized, only requires S + M impls and better yet
//!     it generates those M impls for you with a proc-macro (usually). We're
//!     not looking to make a serialization/deserialization framework so this
//!     isn't a concern for us, but it's interesting to note.
//!   - We don't enforce that all `Enc` impls on `JsonEnc` actually produce
//!     `Json` values. This _could_ be handy, but it's probably a bad idea. Note
//!     that serde's structure _does_ enforce this.
//!
//! So, why not just use `serde`?
//!
//! Well. `serde` has a pretty strict data model that, aiui, requires
//! serialization formats to support things such as maps and all of the Rust
//! primitives/structures (except unions, I guess). This is great since it means
//! that, as a writer of message types, you can (very often) use `serde` with
//! basically no extra work and that there aren't separate types for your actual
//! code and for intermediate serialization/deserialization steps.
//!
//! On the other hand, `serde`'s requirements are a lot stricter than our own
//! which closes the door to wire formats like ProtoBufs which aren't really
//! compatible with `serde`. This seems less than ideal which is why we have the
//! below traits.
//!
//! [^1]: i.e. `JsonEncoding`; a type that this trait is implemented on but is
//!       never actually used for anything.

use core::fmt::Debug;
use core::marker::PhantomData;

// Implementors provide:
//   - T -> Encoded (Infallible)
//   - Encoded -> T (Fallible)
//
// Allowing for a full cycle:
//    [T] -> [Encoded]
//     ^         |
//      \-------/
pub trait Encoding<T: Debug> {
    type Encoded: Debug;
    type Err: Debug;

    // fn encode(message: ControlMessage) -> Result<Self::Encoded, Self::Err>;
    fn encode(message: T) -> Self::Encoded;
    fn decode(encoded: &Self::Encoded) -> Result<T, Self::Err>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TransparentEncoding<T: Debug>(PhantomData<T>);

impl<T: Debug> Encoding<T> for TransparentEncoding<T> {
    type Encoded = T;
    type Err = Infallible;

    fn encode(message: T) -> Self::Encoded {
        Ok(message)
    }

    fn decode(message: &Self::Encoded) -> Result<T, Self::Err> {
        Ok(message.clone())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
// Outer provides:
//   - A -> Outer::Encoded
//   - Outer::Encoded -> A
//
// Inner provides:
//   - B -> Inner::Encoded
//   - Inner::Encoded -> B
//
// Assuming Outer::Encoded = B, we have a full cycle:
//   [A] -> [B / Outer::Encoded] -> [Inner::Encoded]
//    ^                                    |
//    \- <- [Outer::Encoded / B] <--------/
//
// The above contains two encodes and two decodes; this type squashes the above
// into one encode and one decode:
//   [A] -> [Inner::Encoded]
//    ^            |
//     \----------/
pub struct ChainedEncoding<A: Debug, B: Debug, Outer: Encoding<A>, Inner: Encoding<B>>(PhantomData<Outer>, PhantomData<Inner>)
where
    Outer: Encoding<A, Encoded = B>,
    <Inner as Encoding<B>>::Err: Into<<Outer as Encoding<A>>::Err>;


impl<A, B, Outer, Inner> Encoding<A> for ChainedEncoding<A, B, Outer, Inner>
where
    A: Debug,
    B: Debug,
    Outer: Encoding<A, Encoded = B>,
    Inner: Encoding<B>,
    <Inner as Encoding<B>>::Err: Into<<Outer as Encoding<A>>::Err>
{
    type Encoded = <Inner as Encoding<B>>::Encoded;
    type Err = <Outer as Encoding<A>>::Err;

    fn encode(message: A) -> Self::Encoded {
        let b: B = <Outer as Encoding<A>>::encode(message);
        <Inner as Encoding<B>>::encode(b)
    }

    fn decode(message: &Self::Encoded) -> Result<A, Self::Err> {
        let b: B = <Inner as Encoding<B>>::decode(message)?;
        <Outer as Encoding<A>>::decode(b)
    }
}

impl<A, Outer> ChainedEncoding<A, <Outer as Encoding<A>>::Encoded, Outer, TransparentEncoding<<Outer as Encoding<A>>::Encoded>>
where
    A: Debug,
    Outer: Encoding<A>,
    // !: Into<<Outer as Encoding<A>>::Err>,
{
    pub fn new_detached() -> Self {
        Default::default()
    }

    pub fn new(_outer: Outer) -> Self {
        Self::new_detached()
    }
}


impl<A, B, Outer, Inner> ChainedEncoding<A, B, Outer, Inner>
where
    A: Debug,
    B: Debug,
    Outer: Encoding<A, Encoded = B>,
    Inner: Encoding<B>,
    <Inner as Encoding<B>>::Err: Into<<Outer as Encoding<A>>::Err>
{
    pub fn chain_detatched<Z: Debug, NewOuter: Encoding<Z, Encoded = A>>() -> ChainedEncoding<Z, A, NewOuter, Self>
    where
        <Outer as Encoding<A>>::Err: Into<<NewOuter as Encoding<Z>>::Err>
    {
        Default::default()
    }

    pub fn chain<Z: Debug, NewOuter: Encoding<Z, Encoded = A>>(self, _new_outer: NewOuter) -> ChainedEncoding<Z, A, NewOuter, Self>
    where
        <Outer as Encoding<A>>::Err: Into<<NewOuter as Encoding<Z>>::Err>
    {
        Self::chain_detatched()
    }
}

using_std! {
    #[cfg(feature = "json_encoding")]
    pub struct JsonEncoding;

    #[cfg(feature = "json_encoding")]
    impl Encoding for JsonEncoding {
        type Encoded = String;
        type Err = serde_json::error::Error;

        fn encode(message: ControlMessage) -> Result<Self::Encoded, Self::Err> {
            serde_json::to_string(&message)
        }

        fn decode(encoded: &Self::Encoded) -> Result<ControlMessage, Self::Err> {
            serde_json::from_str(encoded)
        }
    }
}

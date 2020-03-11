//! # RPC
//!
//! In picking an encoding scheme for our embedded devices, we have to decide
//! what guarantees we want from our encoding layer (i.e. be able to tell where
//! messages start and end) and what we can assume the layers underneath us
//! provide.
//!
//! Everything written here assumes that our embedded device is using UART: a
//! duplex protocol that does not guarantee data integrity[^parity] or delivery
//! but does (we'll assume) guarantee ordering.
//!
//! In this ramble we kind of end up going through the details for the actual
//! on-the-wire transport (UART, below encoding) and the protocol layer (i.e.
//! error handling, above encoding).
//!
//! [^parity]: UART does frequently use a parity bit but we won't rely on it
//! since we aren't set up to error handle on a byte level. More
//! [here](#but-what-about-uart-parity).
//!
//! ## The Failure Modes
//!
//! As mentioned, for transmission over UART, I think we're mostly concerned
//! about these two error cases:
//!   - flipped bits (fidelity)
//!   - dropped bytes (data loss, i.e. a framing error)
//!
//! Other possible error cases like reordered bytes and duplicated bytes can,
//! _I think_ be safely ignored[^reordered-and-flipped]; they don't seem very
//! likely with UART.
//!
//! [^reordered-and-flipped]: The setup we end up settling on actually *should*
//! be able to handle reordered bytes (checksum fail or length check fail if the
//! sentinel gets moved) and duplicated bytes (again: checksum fail and if it's
//! the sentinel it'll just be like the message was dropped) and both at the
//! same time: just not in a graceful way. Instead of reduplicating or
//! reordering (which we could do if we assigned each datagram a number), we'll
//! just retry, which is, imo, okay for something of this scale/size.
//!
//! ### Protecting against bit flips
//!
//! Flipped bits seem easy enough to protect against: we can preface messages
//! (requests and responses) with a checksum and check this upon receiving a
//! request or a response.
//!
//! Actually handling checksum mismatches (and other errors) is a little
//! trickier; it's easiest to just have the requester just retry but this can't
//! happen unless the requester is somehow _aware_ that the error has happened.
//!
//! #### Actually handling errors
//!
//! I think there are a couple of ways to go about this. One is to use time to
//! guess when errors have happened; i.e. implicit signaling. Another is to
//! actually just tell the requester when something has gone wrong; i.e.
//! explicit signaling.
//!
//! ##### Implicit error signaling: timeouts
//!
//! As an example, one way to have the requester know when to re-try is to
//! decide that if we, the requester, do not get a response within, say, 10ms of
//! sending our request we'll assume that the request wasn't received (or that
//! something went wrong in responding to our request) and to just try again at
//! this point.
//!
//! This seems like it would work but is actually problematic for several
//! reasons! One is that picking such a timeout is *hard*. In order to do so you
//! have to provide an upper bound on response times; otherwise you run the risk
//! of duplicating requests if you don't wait for long enough. Duplicating the
//! requests is not a huge problem but assuming that we won't get a response for
//! a certain request and then later getting that response *will* break our
//! system as it's currently architected.
//!
//! In our case, because our transport is fairly predictable (i.e. we're
//! extremely unlikely to experience randomly high delays) and because the
//! device responding to the requests is fairly deterministic, we could probably
//! come up with timeout values. But to do so would require us to couple
//! [`Controller`] impls pretty tightly to the [`Control`] impls they talk to.
//! Put differently, one implementation of of [`Control`] might have different
//! maximum response times than another and without growing new machinery to
//! communicate this to the requesting device, we wouldn't be able to use the
//! same [`Controller`] with both.
//!
//! Additionally, platforms that use paging (or run OSes) likely just *won't* be
//! able to provide suitable timeout numbers.
//!
//! [`Controller`]: lc3_traits::control::rpc::Controller
//!
//! ##### Explicit error signaling
//!
//! Another way to go about this is to have the responder just tell the
//! requester when to retry. As in: instead of returning the expected response
//! message, return an error.
//!
//! First, we should talk about the two different places an error (framing or
//! flipped bits) can happen:
//!
//! ```text
//!                       /---- A
//!                      v
//!   [Requester] Req -----> [Responder]
//!       ^                       |
//!        \------------ Resp <--/
//!                             ^
//!                              \-------- B
//! ```
//!
//! A: In the request: Before the message gets to the Responder
//! B: In the response: After the message (successfully) gets to the Responder
//!    but before the response gets to the Requester
//!
//! In case A, the responder *knows* something has gone wrong. In case B, the
//! requester *knows* something has gone wrong[^caveat3]. In order for the
//! requester to be able to retry when errors happen we'd want the requester
//! to know about situations like case A as well as case B and that's what this
//! approach does: it has the responder tell the requester when case A happens.
//!
//! Implemented, this means having the requester, when receiving, retry when
//! errors in receiving happen or when an error is received. For the responder
//! this means sending errors when errors happen.
//!
//! The one wrinkle with this approach is the asynchronous [`run_until_event`]
//! response. Under this scheme each thing we receive can be one of three
//! things with a fourth, error during the reception, possibility:
//!   - `run_until_event` response
//!   - current requests' response message
//!   - an explicit error (new! tells us that the req failed, case A)
//!   - an error in reception (case B)
//!
//! The issue is that the last item is ambiguous: we don't know whether the
//! thing we received malformed was a response to our request or a
//! `run_until_event` response.
//!
//! We can't assume that the response that failed to decode was either the
//! request to our response _or_ a `run_until_event` response; if we incorrectly
//! assume something was the response we're waiting on (and retry our request),
//! we run the risk of never resolving the `EventFuture` *and* exploding if/when
//! we get a duplicate response for our request. Alternatively, if we
//! incorrectly assume that the failed response was a `run_until_event`
//! response then we'll just wait on the right response potentially forever. We
//! also have no way to retry the `run_until_event` so that's a problem too.
//!
//! Fortunately, we've got a couple of things going for us. One is that we can
//! frequently infer from context whether or the malformed response should have
//! actually been a [`run_until_event`] response or something else. For example,
//! if we don't have an `EventFuture` out, we can assume it *wasn't* a
//! [`run_until_event`] response. Vice versa with regular responses; if we do
//! have an `EventFuture` out and aren't in the middle of a request, we can
//! assume.
//!
//! In the case where we *are* in the middle of a request and have an
//! `EventFuture` out, we can tell whether or not we were supposed to receive
//! a [`run_until_event`] response by just asking the [`Control`] impl (by
//! calling `get_state` and seeing if we're still running — if we are, we didn't
//! get a [`run_until_event`] response). Note that even if the request where we
//! ask the [`Control`] impl fails in the same way, we're still fine; we can
//! handle the nested faults. From here, we know whether to redo the request or
//! to just wait.
//!
//!
//! The one problem with this, however, is that we don't have a way to recover
//! a borked `run_until_event` response. Unlike other requests we can't just
//! re-request it (it is very much
//! [not idempotent](#wait-but-can-we-just-retry-or-idempotency-is-your-friend))
//! so what can to do?
//!
//! One thing we can do is have the response to `run_until_event` require an
//! acknowledge. On it's own this doesn't solve anything (to be sure that the
//! requester and the responder are consistent, the acknowledge would require
//! an acknowledge and then _that_ acknowledge would need another acknowledge
//! and so on til infinity) but if we make the responder repeatedly send the
//! `run_until_event` response until it gets a response (and have the requester
//! acknowledge whenever it gets one of these), eventually the system will
//! settle.
//!
//! This seems like it should work but the wrinkle is that now there's the
//! question of what should happen when a [`run_until_event`] call has
//! resolved but has yet to be acknowledged by the requester *and* the requester
//! goes and starts a new `run_until_event`. It's like the
//! [reset & batch issue](https://github.com/ut-utp/prototype/issues/48) but
//! worse!
//!
//! For now, as a compromise, I think we can change `EventFuture` to resolve to
//! an `Option` of an `Event` and have that `Option` be `None` specifically in
//! this case (when the actual `EventFuture` gets lost). Note that we still have
//! enough data to confidently resolve the `EventFuture` (i.e. we know for sure
//! that the `EventFuture` has been resolved).
//!
//! [^caveat3]: We're making an assumption here which is that when such failures
//! happen the recipient of the data can tell. This is true so long as not *all*
//! the data is lost which seems like a fair assumption. In order to protect
//! against such situations, I think you'd need a timeout.
//!
//! ##### Wait, but _can_ we just Retry? Or: Idempotency is _Your Friend_
//!
//! In the above we assumed that just that the requester could just retry (i.e.
//! resend its failed request) whenever it did not succeed, but: is this
//! actually a valid assumption?
//!
//! In the above we don't distinguish between errors in _receiving_ requests and
//! errors in _responding_ to requests, but this seems like an important
//! distinction! In the former case, the device didn't get the request so it
//! seems perfectly safe to retry: no one will actually really know we tried in
//! the first place and if all calls on [`Control`] are blocking and retry until
//! they succeed, there's no risk of the request message being out of date or
//! being received and processed by the device twice.
//!
//! But, if the error happens when we, the requester, are receiving the
//! response, that means the device already saw a (possibly — remember, the
//! message we have trouble receiving could very well be a message saying that
//! there was an error while requesting) valid request so when we retry that
//! message will be processed a second time!
//!
//! So, it turns out, that in most cases, this should not be a problem because
//! the functions in the [`Control`] interface are (mostly)
//! [idempotent](https://en.wikipedia.org/wiki/Idempotence). Put another way,
//! this means that you can call functions on [`Control`] multiple times (with
//! the same arguments and without doing anything else to the Control impl in
//! between) and it'd be the same as if you just called the function once.
//!
//! For some functions like the getters on [`Control`] (i.e. [`get_pc`]) this is
//! fairly obviously true; calling a getter multiple times should yield the same
//! value in the absence of things that can change the value[^caveat]. This is
//! true for setters as well, it turns out[^caveat2].
//!
//! Things like [`set_memory_watchpoint`] are careful in their handling of
//! already set watchpoints (they'll just return the index they already occupy),
//! so such functions will work fine (this includes [`run_until_event`]).
//!
//! The problematic functions are [`unset_memory_watchpoint`] and
//! [`unset_breakpoint`] as well as [`step`] and [`start_page_write`] and
//! [`end_page_write`]. The last two are problematic because duplicate calls to
//! them will *not* return the same thing. But, because the [porcelain function]
//! (lc3_traits::control::load::load_memory_dump) for loading memory should
//! just retry in such situations (or user code can retry), we'll call this
//! okay.
//!
//! For `step`, the failure mode is stepping multiple times when we were asked
//! to step only once. This is problematic.
//!
//! On duplicate calls, `unset_memory_watchpoint` and `unset_breakpoint`
//! can return errors instead of accurately reporting that a
//! breakpoint/watchpoint has been successfully removed. We could remedy this
//! by caching the last removed breakpoint/watchpoint and returning success if
//! that breakpoint/watchpoint tries to be removed again but this is not a
//! perfect solution (since we can't distinguish between legitimate duplicate
//! attempts and ones due to transmission errors). Another way to solve this is
//! to have the responder cache the last response and have the requester
//! explicitly signal when it is retrying; the responder can then go send the
//! cached response it has (if the last request it has matches the retried
//! request) rather than actually running the function again.
//!
//! The solution described above actually ends up making every function
//! unproblematically retry-able but at the cost of adding a `is_a_retry` field
//! to every request.
//!
//! For now, we'll say the above failures are unideal but acceptable and in the
//! event that these actually arise in practice, we'll revisit this and
//! implement the above strategy (TODO).
//!
//! [`Control`]: lc3_traits::control::Control
//! [`get_pc`]: lc3_traits::control::Control::get_pc
//! [`set_memory_watchpoint`]: lc3_traits::control::Control::set_memory_watchpoint
//! [`run_until_event`]: lc3_traits::control::Control::run_until_event
//!
//! [`unset_memory_watchpoint`]: lc3_traits::control::Control::unset_memory_watchpoint
//! [`unset_breakpoint`]: lc3_traits::control::Control::unset_breakpoint
//! [`step`]: lc3_traits::control::Control::step
//! [`start_page_write`]: lc3_traits::control::Control::start_page_write
//! [`end_page_write`]: lc3_traits::control::Control::end_page_write
//!
//! [^caveat]: So, actually, it turns out we can continue stepping in between
//! these duplicate requests but that is _fine_ because when we're executing
//! we never guaranteed exactly when your commands would be processed relative
//! to the ongoing execution _anyways_. If you want such guarantees, send your
//! commands when the machine is _paused_.
//!
//! [^caveat2]: As with getters, this is problematic[^caveat] if we're currently
//! executing (and actually worse with setters since you could, for example, set
//! the PC, execute a bit, and then set the PC again) but still _okay_ since we
//! don't really guarantee behavior when doing things while executing anyways.
//! Also, implementors should block if they get an error and retry anyways (as
//! in not continue stepping) so this is all actually a moot point; behavior is
//! predictable and all is well.
//!
//! #### But what about UART parity?
//!
//! As mentioned, UART using devices often send and check for a parity bit but
//! we won't use this, for the following reasons:
//!   - We are not set up to actually handle errors in words at the byte level.
//!      * If we (or the UART hardware) see that the parity bit is off for a
//!        particular word, we don't have a way to signal the sending device to
//!        resend *just that word*.
//!         + Adding a mechanism to allow us to ask for individual bytes to be
//!           re-sent would require us to either number the bytes (doable) and
//!           then ask for a byte, by number, to be re-sent (I think this is
//!           a common UDP transmission strategy?) or to synchronize the
//!           receiver/transmitter on every byte (expensive, not really doable)
//!           and then to re-request the "current byte" that the system is on.
//!      * Our [`Transport`] infrastructure isn't really set up for this. Our
//!        current traits are designed for [`Transport::get`] to _only receive
//!        things_ and not send and receive as the above scheme (i.e. sending
//!        re-send requests for bytes where the parity doesn't match). We could
//!        still write an impl of [`Transport`] that does such a thing but it
//!        complicates things (i.e what do we do about things that are already
//!        queued for sending?).
//!      * We want to optimize for the common case (no errors) and we're okay
//!        with selling out our worst case (i.e. making it more expensive). The
//!        trade-off with *not* using byte level parity data is that situations
//!        with errors are more expensive (they require a full re-send) but this
//!        allows us to perform less checks on the (we hope) common case and
//!        send fewer bits.
//!
//! Also, a single parity bit for each byte seems like a wimpier data integrity
//! scheme than having a checksum on your entire (multi-byte) message. The
//! latter actually uses _more_ bits than the former but it is resilient to
//! multiple bit flips in the same byte. On the other hand, single parity bits
//! can tolerate more _overall_ bit flips (assuming your checksum is tiny — say
//! a byte ­— compared to your message). So, I guess it really depends on the
//! distribution of your bit flips.
//!
//! [`Transport`]: lc3_traits::control::rpc::Transport
//! [`Transport::get`]: lc3_traits::control::rpc::Transport::get
//!
//!
//! ### Protecting against dropped bytes (aka framing errors)
//!
//! Dropped bytes[^dropped-bits] are problematic but before we discuss why, we
//! need to talk about framing.
//!
//! [^dropped-bits]: Note that we don't discuss dropped bits because UART
//! operates at the word level; if individual bits get dropped we'll either
//! drop the entire byte or just get a byte with wrong data (a bit flip error).
//!
//! #### Framing, or: Where to Start and End
//!
//! For our purposes, all UART provides us with is a stream of bytes. From this
//! we need to extract (multi-byte) messages and one of the challenges in doing
//! so is figuring out which bytes correspond to a message, or, put differently
//! where messages start and end. This is the question of framing.
//!
//! One very simple framing strategy is to have _fixed length messages_: if your
//! message length is `n`, the first `n` bytes will correspond to a message, the
//! next `n` to the next, and so on. Nice and simple.
//!
//! This works — but at some cost. If your messages aren't fixed size, and you
//! want to use this scheme you'll need to pick the largest message size you
//! have and _make_ that your fixed size; all messages that are smaller get
//! padded.
//!
//! If your messages aren't too varied in size, the above is a viable strategy
//! — however this frequently isn't the case.
//!
//! Another option is to explicitly specify each message's length by prefixing
//! the message with its length.
//!
//! This is less wasteful than picking a fixed message length and it too will
//! work — provided you never drop any bytes at all. Consider the following
//! example where messages are 5 elements long and of the format
//! `[a-z] [a-z] : [0-9] [0-9]`:
//!
//! ```text
//! Sent:   | a b : 3 2 | m e : 2 5 | u s : 4 2 |
//!           | | | | |   | | | | |   | | | | |
//!           | | | | x   | | | | |   | | | | |
//!           v v v v     v v v v v   v v v v v
//!           a b : 3     m e : 2 5   u s : 4 2
//!    |
//!     \->   a b : 3 m e : 2 5 u s : 4 2
//!
//! Recv:   | a b : 3 m | e : 2 5 u | s : 4 2   |
//!               |           |           |
//!               v           v           v
//!            Error!       Error!      Error!
//! ```
//!
//! In the above, only one bit was lost, but rather than just compromise the
//! message that that one bit was part of, the following messages are also
//! compromised! Even though the bytes in the consecutive messages were all
//! transmitted successfully, we were unable to decode the messages because we
//! matched the bytes to messages incorrectly, causing the message parsing to
//! fail[^detecting-framing-errors].
//!
//! [^detecting-framing-errors]: In this toy example, it is very possible to
//! detect framing errors since there are many possible invalid states. In an
//! actual format it might not be so possible, but the point is that it depends
//! on your format (as in, it breaks the clean separation we're striving for).
//! More [here](a-detour-about-detecting-and-recovering-from-framing-errors).
//!
//! ##### A detour about detecting (and recovering from) framing errors
//!
//! In the above toy example, it's very easy to detect framing errors since
//! there are many possible byte combinations that are invalid. An encoding that
//! offers good compression will not have this property; it aims to make use of
//! as many of the possible byte combinations as possible to send _fewer_ bytes.
//!
//! All this is to say that we might not even _detect_ that a framing error has
//! occurred, in which case we'd simply continue processing messages but _every
//! message after the framing error would not match the real message_. This is
//! bad.
//!
//! Also worth mentioning is how long it takes to _recover_ from framing errors.
//! If we're unable to detect that a framing error has happened and continue
//! processing messages oblivious, the only way for us to recover would to for
//! us to experience more framing errors until we're shifted over by one entire
//! message length.
//!
//! If we _do_ detect that something has gone wrong (i.e. a message that doesn't
//! parse) we can try to do better. For example, we could start to throw away
//! bytes and try to reparse until it succeeds and _assume_ that the successful
//! parse means that we're aligned again. Again, this is heavily dependent on
//! the format and characteristics of the messages being sent; if the messages
//! are sufficiently long or have a relatively small number of error states,
//! then it's entirely possible that we realign ourselves incorrectly.
//!
//! #### Framing in a fallible world
//!
//! So, it's clear that we can't use fixed size or length prefix framing schemes
//! on transports that can drop chunks of data and that recovering from framing
//! errors using the data we *do* get _at best_ requires support from the data
//! we're sending and _at worst_ just isn't reliable.
//!
//! So, what do we do?
//!
//! As with handling errors, another way is to [be explicit]
//! (#explicit-error-signaling). Rather than try to guess at where frames start
//! and end by counting bytes and seeing what successfully parses, we can add
//! 'markers' to the bytes we transmit indicating the framing.
//!
//! For example, we could terminate every message with a _sentinel_ (i.e. the
//! way C style `NULL` terminated strings do) which we'd then watch for when
//! receiving bytes:
//!
//! ```text
//! a b : 3 2 ø m e : 2 5 ø u s : 4 2 ø
//! ```
//!
//! If the bytes are dropped, the sentinel always lets us know where to start
//! reading the next message so we only lose message the bytes belong to:
//!
//! ```text
//! a b : 3 2 ø m e : 2 5 ø u s : 4 2 ø
//! | | | | | | | | | | | | | | | | | |
//! | | | | x | | | | | | | | | | | | |
//! v v v v   v v v v v v v v v v v v v
//! a b : 3   ø m e : 2 5 ø u s : 4 2 ø
//!     |           |           |
//!     v           v           v
//!   Error!     Success     Success
//! ```
//!
//! If we lose the sentinel byte then we lose the message _and_ the next message
//! but will still recover afterwards.
//!
//! It's worth noting that this is somewhat similar to what UART does with
//! start and stop bits.
//!
//! Also note that the sentinel obviates the need for explicit length prefixing,
//! though it can be useful to include anyways as a way to detect situations
//! where the sentinel is dropped sooner.
//!
//! The tradeoff with sentinels are that:
//!   - they add to the number of bytes that must be transmitted
//!      + in the above, they require one additional byte which comes out to
//!        20% overhead
//!      + in situations with larger messages (i.e. we've got ~40 byte requests)
//!        this is not very significant
//!      + of course, it is also possible to amortize this; for example, send
//!        a sentinel every 4 or 8 messages, etc.
//!   - they take away from the pool of values that can be used in messages
//!
//! The latter point is an important one. The sentinel value can't be used in
//! the actual message you are sending! In the above that is fine since the
//! message format doesn't allow for `ø`s anyways. In C style strings, `'ø'` or
//! `NULL` aren't allowed in strings either so it isn't a problem. Put
//! differently, sentinel based framing schemes require cooperation from the
//! data being transmitted!
//!
//! This is problematic for us since we'd like to keep the message format
//! decoupled from the encoding *and* since many of our messages transmit, for
//! example, `Word`s (i.e. unsigned 16 bit numbers) it's very difficult for us
//! to specify any sentinel at all since a `Word`'s valid encodings span all
//! possible byte values for two bytes. This, and situations like it, aren't
//! on their own a deal breaker. There is nothing that requires a _single_ byte
//! sentinel; we could switch to a multi-byte sentinel instead. But this, too,
//! is a tradeoff. A multi-byte encoding steals fewer valid values from the
//! encoding but it also increases the size of the messages and thereby
//! increases the odds that a message will have one or more bytes dropped
//! (assuming an independent byte drop rate). Also, you want to pick your
//! sentinel such that it's easy to align on without error; making your sentinel
//! longer is actually at odds with this.
//!
//! So what to do now?
//!
//! Sentinels are an attractive framing technique since they have relatively
//! low overhead and can handle arbitrary bytes dropping. But they seem to
//! require cooperation from the data being transmitted which is a non-starter
//! for us, so it seems like we're back where we started.
//!
//! #### COBS to the Rescue!
//!
//! What we really want is a way to ensure that the data we're sending doesn't
//! contain a particular value without actually imposing any restrictions on the
//! data we're sending.
//!
//! It turns out this is totally a thing that is possible and it's called [byte
//! (or bit) stuffing](wpi-stuf)!
//!
//! One such implementation of this is [Consistent Overhead Byte Stuffing (aka
//! _COBS_)](COBS); COBS works (approximately) by dividing up your data (which
//! is allowed to have zeros) into chunks wherever the zeros are (or after
//! 254 non-zero bytes — the end of your data also has an implicit zero):
//!
//! ```text
//!  /-- 'a' 'b' 'c' 0 'h' 'e' 'l' 'l' 'o' 0 1 0 1 .. 255 _0_
//!  |
//!  |
//!  \-> | 'a' 'b' 'c' | 'h' 'e' 'l' 'l' 'o' | 1 | 1 .. 254 | 255 |
//! ```
//!
//! Now, prepend each chunk with length of the chunk + 1. Or, put another way,
//! put the index (+1) of the chunk's zero in front of each chunk:
//!
//! ```text
//! | 'a' 'b' 'c' | 'h' 'e' 'l' 'l' 'o' | 1 | 1 .. 254 | 255 |
//!
//! |4| 'a' 'b' 'c' |6| 'h' 'e' 'l' l' 'o' |2| 1 |254| 1 .. 254 |2| 255
//! ```
//!
//! Note that the index is guaranteed to be ≤ 255 since each chunk can be at
//! most 254 bytes long. Also note that a length of 255 is a special case
//! corresponding to 254 non-zero bytes and *no zero*. If you have actually have
//! 254 non-zero bytes followed by a zero it'll be treated as two chunks, the
//! latter being an 'empty' chunk.
//!
//! The one slightly problematic case seems to be that you'll get an extra
//! trailing zero if your data ends with a sequence of non-zero bytes that is
//! less than 254 elements long. But this isn't too difficult to remedy; as a
//! simple fix (that also has other useful properties ­— like letting us size
//! buffers) we can go back to prepending messages with the total length.
//!
//! COBS is pretty nifty because of just how low overhead it is. Each zero that
//! is replaced incurs no extra overhead since the length that it's effectively
//! 'substituted' with is also a single byte. 254 byte runs of non-zeros do
//! incur a single byte of overhead, however.
//!
//! It's also worth mentioning that nothing says the delimiter you use has to be
//! a zero; the COBS approach can be used with any arbitrary single-byte value
//! (and indeed; the [Rust cobs crate](https://docs.rs/cobs/0.1.4/cobs/) allows
//! you to [specify a sentinel value](cobs-sentinel)). It's unintuitive but to
//! minimize overhead you'd actually want a sentinel value that _is_ used
//! frequently in the actual message bytes (alternatively if you're sending
//! less ≤ 254 bytes at a time, it is irrelevant).
//!
//! Another consideration for picking a sentinel value may be the likelihood of
//! detecting transmission bit errors in that sentinel. For example, if you've
//! got a UART setup where the baud rates for the two devices are just on the
//! edge of what's workable, it's possible that you could get bits that are
//! repeated and replace the next bit; i.e. this:
//!
//! ```text
//! ---   ---            ---   ---------
//!    ___   ____________   ___
//!     |  0  1  2  3  4  5  6  7  |  |
//!     v                          v  v
//!   Start                     Stop Bits
//!
//!  -> 0b10000101
//! ```
//!
//! could get read as `0b11000101` or `0b00000101` or `0b01000101` if one pulse
//! is a little short (i.e due to capacitance in the wire) or if things are
//! delayed a bit and things are aligned just so.
//!
//! This isn't actually a _problem_ since we'll still recover even if our frame
//! marker goes missing for a frame (it's also already a pretty extreme edge
//! case), but it's worth mentioning as a consideration in picking a sentinel
//! value. Regardless, 0 seems like a good choice.
//!
//! [COBS]: https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing
//! [wpi-stuf]: https://web.cs.wpi.edu/~rek/Undergrad_Nets/B07/BitByteStuff.pdf
//! [cobs-sentinel]: https://docs.rs/cobs/0.1/cobs/fn.decode_in_place_with_sentinel.html
//!
//! #### What about UART framing errors?
//!
//! [As with dropped bits](#what-about-uart-parity), UART has an analogue for
//! framing errors at the word level that it operates at. For UART these errors
//! happen when words aren't followed by the right stop bits, I believe.
//!
//! Since our messages are larger than single words, this alone isn't a
//! sufficient message framing mechanism.
//!
//! [As with UART parity](#what-about-uart-parity), we could use these errors to
//! terminate an attempt early, but instead (in the interest of simplicity)
//! we'll just let such errors bubble up into the framing/checksum errors that
//! we already handle at the message level.
//!
//! ## Altogether Now
//!
//! Ultimately, actually implementing a error tolerant rpc system as described
//! here involves three main components, I think:
//!
//! 1) Modifying the [`Controller`], [`Device`], and
//!    [`RequestMessage`]/[`ResponseMessage`] types to support the explicit
//!    error reporting mechanisms described above and to actually retry when
//!    errors happen in the way described above.
//!
//!    - To test that this machinery works, we can 'send' [`Control`] calls
//!      across an rpc setup that uses a mock transport. This mock transport
//!      can emulate a fallible transport by occasionally turning some of the
//!      messages sent across it into Errors. The [`Controller`] and [`Device`]
//!      should continue relaying calls across; from the perspective of the
//!      user of the [`Controller`] and the [`Control`] impl that the [`Device`]
//!      wraps, no messages should be lost. However, the two may differ in their
//!      estimation of the _number_ of calls sent across; this is okay and
//!      expected.
//!
//! 2) Making the UART Transport.
//!
//!    - This will have to ensure the uart instance we're using in in whatever
//!      configuration (i.e. 8b, 1 stop, no parity) we settle on.
//!    - This will also be responsible for framing. That means it will have to
//!      be aware of any length-prefixing we do + any sentinel we use. This
//!      makes for an acceptable API, I think. This layer doesn't need to be
//!      aware of COBS; just that, i.e. 0 is the sentinel. Both the length
//!      prefixing and the exact sentinel used can be configurable.
//!    - We should have the length be _inclusive_ of the sentinel.
//!    - This API is poll based so what should probably happen is that every
//!      time [`Transport::get`] is called, this checks the current buffer of
//!      UART bytes received from the UART and updates a state machine that
//!      probably looks something like this:
//!       * InitialState:
//!          + On new bytes, take the first as the length; go to
//!            `ReceivingMessage`.
//!          + Return `Err(NoMessage)` (no messages to process)
//!       * ReceivingMessage:
//!          + On new bytes, copy into the internal buffer until we run out of
//!            bytes or hit the sentinel or the count hits zero.
//!             - If we hit the sentinel *and* the count is zero:
//!                * Try to decode the message and if that succeeds, return
//!                  `Some(msg)`. (nvm; actually just return a reference to the
//!                  internal buffer or something)
//!                * Transition to `PendingMessage` (which has a frozen buffer)
//!                * Return the framed message; `Some(_)`.
//!             - If we hit the sentinel *and* the count is greater than zero:
//!                * Byte underrun!
//!                * Discard the buffer; switch to `InitialState`.
//!                * Return `LengthMismatch`
//!             - If the count is zero but we didn't hit the sentinel:
//!                * Byte overrun! (probably a dropped sentinel)
//!                * Discard the buffer; switch to `AwaitingSentinel`.
//!                * Return `LengthMismatch`
//!             - Otherwise, (i.e. if we've run out of bytes):
//!                * Update the count, go back to the same state.
//!                * Return `MessageInProgress`
//!         + This is maybe subtle but this just _won't_ store things from the
//!           fifo it has a reference to (behind a bare metal Mutex) beyond the
//!           current message (but it will grab them so the fifo doesn't
//!           overflow).
//!       * AwaitingSentinel:
//!          + Throws away any bytes other than the sentinel.
//!          + Upon getting a sentinel, resets back to `InitialState`.
//!          + This is where we get when we're waiting to realign/recover from
//!            a framing problem.
//!       * AwaitingReset:
//!          + This contains a buffer containing a framed message.
//!          + Immediately switches back to `IntialState`; should not produce a
//!            return value.
//!    - Send will just prefix with a length and dump into the Uart send FIFO.
//!       * As an optimization this can also enable uart_tx interrupts; the
//!         interrupt can then go disable itself if it ever runs and doesn't
//!         have data to send.
//!    - The error types probably look something like this:
//!      ```rust,ignore
//!      enum UartTransportRecvError {
//!          NoMessage,
//!          MessageInProgress,
//!          /// Overrun or underrun
//!          LengthMismatch { overrun: bool, expected: usize, got: usize },
//!      }
//!
//!      enum UartTransportSendError {
//!          UartFifoFull,
//!          ???
//!      }
//!      ```
//!    - To support cooperation with interrupts, we could have an AtomicBool
//!      that gets set in interrupts and is checked to see if we have new data
//!      in [`Transport::get`].
//!    - Unlike the error handling thing in step 1, this should be fairly easy
//!      to unit test; we shouldn't need to make any mocks or fake [`Control`]
//!      impls.
//!
//! 3) The Encodings.
//!
//!    - Rather than just have one, I think we continue the middleware-esque
//!      approach that's worked well for us:
//!
//!    1) Checksumming can be one layer.
//!       + It takes an inner encoding/decoding.
//!       + The Decoding uses the first <word> as the checksum and then checks
//!         if the rest of the slice it was given matches the checksum.
//!          * As with the transport, the particulars can be configurable,
//!            including what checksum to use/how many bytes long it is, etc.
//!       + If the checksum doesn't match, terminate without calling the inner
//!         decoding. If it does match, try to call the inner decode.
//!       + The Decoding error type should look something like this:
//!         ```rust,ignore
//!         enum ChecksumDecodingError<Inner: Debug> {
//!             ChecksumMismatch { expected: <tbd>, got: <tbd> },
//!             InnerDecodingError(Inner),
//!         }
//!         ```
//!       + Encode can work the same way, but in reverse; call the inner encode,
//!         checksum it, and then prepend the checksum onto the inner encode's
//!         result.
//!       + Managing to do this without a heap will be interesting.
//!
//!    2) The actual COBS encoded `postcard` thing can be another layer.
//!       + Ideally, COBS would be it's own layer, but `postcard` internally
//!         uses a middleware like thing that's somewhat similar to what we do
//!         (but only for serialize, not for deserialize) so for simplicity and
//!         for the sake of symmetry with the decode, we'll just have this be
//!         one layer.
//!       + Unlike basically everything else, this should actually be fairly
//!         straightforward, maybe.
//!
//!    - Testing this should be even simpler than testing the transport.
//!
//! [`Device`]: lc3_traits::control::rpc::Device
//! [`RequestMessage`]: lc3_traits::control::rpc::RequestMessage
//! [`ResponseMessage`]: lc3_traits::control::rpc::ResponseMessage
//!
//! ## Multiplexing UART for [`Control`] and for [`Input`]/[`Output`]
//!
//! Another fun bit of complexity that our system has is that we want to
//! multiplex UART to also be used for the [input](`Input`) and
//! [output](`Output`) peripherals too.
//!
//! For our actual transmission protocol the changes aren't too drastic. We can
//! simply prepend each kind of message (input/output or control) with a symbol
//! indicating what the type of the message is:
//!
//! ```plain
//! [Msg Type]
//!   |
//!   | -> C. Resp: [C. Resp Sym] [Length] [Checksum] [ ... data bits ... ] [0]
//!   | <- C. Req:  [C. Req  Sym] [Length] [Checksum] [ ... data bits ... ] [0]
//!   |
//!   | -> Input:   [Input  Sym] [Checksum] [Single Character]
//!   | <- Output:  [Output Sym] [Checksum] [Single Character]
//! ```
//!
//! ### Where to multiplex?
//!
//! We have a couple of options for where we choose to actually separate the
//! different kinds of data and do I/O for the [`Input`]/[`Output`] impls.
//!
//! The obvious way (I think) is to throw this additional logic into the
//! `Transport`; it already gets all the incoming bytes and polls for new
//! messages frequently. The downsides to adding this logic to the `Transport`
//! would are:
//!   - The `Transport` now need to hold a reference to the [`Input`] impl,
//!     while the [`Input`] impl is used by the simulator.
//!      + This is gross but not a dealbreaker; it means implementing [`Input`]
//!        on an immutable reference to some type and using interior mutability
//!        for everything.
//!   - The proper function of the [`Input`] impl is contingent on a `Transport`
//!     being up and running and holding a reference to the [`Input`] impl;
//!     something that would not be obvious to anyone who doesn't read the fine
//!     print.
//!
//! It's also worth noting that the [`Output`] impl would *not* be able to go
//! through the `Transport`. Instead it'd need it's own reference to whatever
//! buffer ultimately gets transmitted, so that's an additional bit of
//! asymmetry.
//!
//! //////////////////////////////////// TODO!!! Finish this.
// ! Note: have the `Output` interrupt_ready function be sourced from its' fifo's
// ! `is_full`.
//!
//! Another option is to do this processing at the point
//!
//! ### [`Input`]/[`Output`] Implementations
//!
//! We'd need make an [`Input`] implementation that has functions that allow us
//! (rather than a device or actual hardware) to provide the inputs. So, pretty
//! much just the `InputShim` but for `#![no_std]`. This also does not have to
//! be generic at all, unlike the `InputShim`.
//!
//!
//!
//! Additionally, because the transport (assuming we still want the transport to
//! be the only thing that )
//!
//! Our transport layer above would need to:
//!   - Take a reference to a grow hooks and new states in its
//! state machine
//!
//! [`Input`]: lc3_traits::peripherals::Input
//! [`Output`]: lc3_traits::peripherals::Output
//!
//! ## Things to investigate in the future: (TODO)
//!  - Using DMA to transfer received messages into <wherever the transport goes
//!    to look for new messages> + having a line break interrupt actually go
//!    trip the flag that has the [`Controller`] go and actually try to process
//!    the received bytes.
//!  - Error correcting codes instead of just checksums.
//!  - Retrying individual bytes instead of entire messages.
//!  - Maybe think about what a non-poll based API would look like.
//!  - Layering on compression below COBS.
//!  - Grow a status field on the Transport layer that tells us how many things
//!    have to be retried/dropped.
//!  - The Controller/Device and Transport layer should add logging letting us
//!    know when things are getting dropped/retried!


pub mod encoding;
pub mod transport;

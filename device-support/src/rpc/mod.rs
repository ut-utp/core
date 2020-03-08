//! TODO!
//!
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
//! on the wire transport (UART, below encoding) and the protocol layer (i.e.
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
//! _I think_ be safely ignored; they don't seem very likely with UART.
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
//!                             \
//!                              \-------- B
//! ```
//!
//! A: In the request: Before the message gets to the Responder
//! B: In the response: After the message (successfully) gets to the Responder
//!    but before the response gets to the Requester
//!
//! In case A, the responder *knows* something has gone wrong. In case B, the
//! requester *knows* something has gone wrong[^caveat3]. In order for the
//! requester to be able to retry when errors happen we'd want to the requester
//! to know about situations like case A as well as case B and that's what this
//! approach does: it has the responder tell the requester when case A happens.
//!
//! Implemented, this means having the requester, when receiving, retry when
//! errors in receiving happen or when an error is received. For the responder
//! this means sending errors when errors happen.
//!
//! The one wrinkle with this approach is the asynchronous [`run_until_event`]
//! response. Under this scheme each thing we receive can be one of three
//! things with a fourth, error during the reception possibility:
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
//! ##### Wait, but can we just Retry? Or: Idempotency is _Your Friend_
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
//! [`Transport::send`]: lc3_traits::control::rpc::Transport::get
//!
//!
//! ## Things to investigate in the future:
//!  - Using DMA to transfer received messages into <wherever the transport goes
//!    to look for new messages> + having a line break interrupt actually go
//!    trip the flag that has the [`Controller`] go and actually try to process
//!    the received bytes.


pub mod encoding;
pub mod transport;
